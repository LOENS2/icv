use crate::data::v4l::CaptureRequest;
use log::debug;
use std::error::Error;
use std::os::fd::{AsRawFd, RawFd};
use tokio::io::unix::AsyncFd;
use tokio::sync::mpsc;
use tokio_linux_video::Device;
use tokio_linux_video::types::{BufferType, Memory, PixFormatMplane};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct V4l2RequestBuffers {
    count: u32,
    type_: u32,
    memory: u32,
    capabilities: u32,
    flags: u8,
    reserved: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct V4l2Plane {
    bytesused: u32,
    length: u32,
    m: V4l2PlaneM,
    data_offset: u32,
    reserved: [u32; 11],
}

#[repr(C)]
#[derive(Copy, Clone)]
union V4l2PlaneM {
    mem_offset: u32,
    userptr: libc::c_ulong,
    fd: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct V4l2Buffer {
    index: u32,
    type_: u32,
    bytesused: u32,
    flags: u32,
    field: u32,
    timestamp: [u8; 8],
    timecode: [u8; 16],
    sequence: u32,
    memory: u32,
    m: V4l2BufferM,
    length: u32,
    reserved2: u32,
    request_fd: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
union V4l2BufferM {
    offset: u32,
    userptr: libc::c_ulong,
    planes: *mut V4l2Plane,
    fd: i32,
}

nix::ioctl_readwrite!(vidioc_reqbufs, b'V', 8, V4l2RequestBuffers);
nix::ioctl_readwrite!(vidioc_querybuf, b'V', 9, V4l2Buffer);
nix::ioctl_readwrite!(vidioc_qbuf, b'V', 15, V4l2Buffer);
nix::ioctl_readwrite!(vidioc_dqbuf, b'V', 17, V4l2Buffer);
nix::ioctl_write_ptr!(vidioc_streamon, b'V', 18, u32);
nix::ioctl_write_ptr!(vidioc_streamoff, b'V', 19, u32);

struct MappedPlane {
    ptr: *mut libc::c_void,
    len: usize,
}

struct MappedBuffer {
    planes: Vec<MappedPlane>,
    v4l2_planes: Vec<V4l2Plane>,
}

unsafe impl Send for MappedBuffer {}

struct FdReady(RawFd);

impl AsRawFd for FdReady {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

/// Mplane-safe MMAP capture loop.
///
/// Works around `linux-video-core` 0.1.1's broken MPLANE mmap handling (it uses
/// single-plane `buffer.length`/`m.offset`; for `VIDEO_CAPTURE_MPLANE` the
/// kernel expects `length = num_planes` and per-plane data in `m.planes`).
pub(super) async fn capture_thread(
    device_path: &str,
    mut rx: mpsc::Receiver<CaptureRequest>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let device = Device::open(device_path).await?;

    let format = device.format(BufferType::VideoCaptureMplane).await?;
    debug!("Format: {format}");

    let num_planes = format
        .try_ref::<PixFormatMplane>()
        .map(|p| p.plane_fmt().len().max(1) as u32)
        .unwrap_or(1);

    let fd = device.as_raw_fd();
    let buf_type = BufferType::VideoCaptureMplane as u32;
    let mem_type = Memory::Mmap as u32;
    let buf_count = 2u32;

    let mut req = V4l2RequestBuffers {
        count: buf_count,
        type_: buf_type,
        memory: mem_type,
        capabilities: 0,
        flags: 0,
        reserved: [0; 3],
    };
    unsafe { vidioc_reqbufs(fd, &mut req) }.map_err(|e| format!("REQBUFS failed: {e}"))?;
    if req.count < 2 {
        return Err(format!("Not enough buffers: {}", req.count).into());
    }

    let mut buffers: Vec<MappedBuffer> = Vec::with_capacity(req.count as usize);
    for i in 0..req.count {
        let mut v4l2_planes = vec![
            V4l2Plane {
                bytesused: 0,
                length: 0,
                m: V4l2PlaneM { mem_offset: 0 },
                data_offset: 0,
                reserved: [0; 11],
            };
            num_planes as usize
        ];

        let mut buf: V4l2Buffer = unsafe { std::mem::zeroed() };
        buf.index = i;
        buf.type_ = buf_type;
        buf.memory = mem_type;
        buf.length = num_planes;
        buf.m = V4l2BufferM {
            planes: v4l2_planes.as_mut_ptr(),
        };

        unsafe { vidioc_querybuf(fd, &mut buf) }
            .map_err(|e| format!("QUERYBUF({i}) failed: {e}"))?;

        let mut mapped_planes = Vec::with_capacity(num_planes as usize);
        for plane in &v4l2_planes {
            let offset = unsafe { plane.m.mem_offset } as libc::off_t;
            let len = plane.length as usize;
            let ptr = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    len,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    fd,
                    offset,
                )
            };
            if ptr == libc::MAP_FAILED {
                return Err(format!("mmap failed: {}", std::io::Error::last_os_error()).into());
            }
            mapped_planes.push(MappedPlane { ptr, len });
        }

        unsafe { vidioc_qbuf(fd, &mut buf) }.map_err(|e| format!("QBUF({i}) failed: {e}"))?;

        buffers.push(MappedBuffer {
            planes: mapped_planes,
            v4l2_planes,
        });
    }

    let mut stream_type = buf_type;
    unsafe { vidioc_streamon(fd, &mut stream_type) }
        .map_err(|e| format!("STREAMON failed: {e}"))?;

    let async_fd = AsyncFd::new(FdReady(fd))?;

    let result = async {
        while let Some(request) = rx.recv().await {
            match request {
                CaptureRequest::Capture { response } => {
                    let mut dq_planes = vec![
                        V4l2Plane {
                            bytesused: 0,
                            length: 0,
                            m: V4l2PlaneM { mem_offset: 0 },
                            data_offset: 0,
                            reserved: [0; 11],
                        };
                        num_planes as usize
                    ];
                    let mut dq: V4l2Buffer = unsafe { std::mem::zeroed() };
                    dq.type_ = buf_type;
                    dq.memory = mem_type;
                    dq.length = num_planes;
                    dq.m = V4l2BufferM {
                        planes: dq_planes.as_mut_ptr(),
                    };

                    loop {
                        let mut guard = async_fd.readable().await?;
                        match unsafe { vidioc_dqbuf(fd, &mut dq) } {
                            Ok(_) => {
                                guard.clear_ready();
                                break;
                            }
                            Err(nix::errno::Errno::EAGAIN) => {
                                guard.clear_ready();
                                continue;
                            }
                            Err(e) => return Err(format!("DQBUF failed: {e}").into()),
                        }
                    }

                    let index = dq.index as usize;
                    if index >= buffers.len() {
                        return Err(format!("DQBUF returned invalid buffer index {index}").into());
                    }
                    let total: usize = dq_planes
                        .iter()
                        .zip(buffers[index].planes.iter())
                        .map(|(plane, mapped)| {
                            let used = if plane.bytesused > 0 {
                                plane.bytesused as usize
                            } else {
                                mapped.len
                            };
                            used.min(mapped.len)
                        })
                        .sum();
                    let mut bytes = Vec::with_capacity(total);
                    let buf_state = &buffers[index];
                    for (mapped, plane) in buf_state.planes.iter().zip(dq_planes.iter()) {
                        let used = if plane.bytesused > 0 {
                            plane.bytesused as usize
                        } else {
                            mapped.len
                        };
                        let used = used.min(mapped.len);
                        let slice =
                            unsafe { std::slice::from_raw_parts(mapped.ptr as *const u8, used) };
                        bytes.extend_from_slice(slice);
                    }

                    // requeue
                    let qb = &mut buffers[index];
                    dq.index = index as u32;
                    dq.type_ = buf_type;
                    dq.memory = mem_type;
                    dq.length = num_planes;
                    dq.m = V4l2BufferM {
                        planes: qb.v4l2_planes.as_mut_ptr(),
                    };
                    for (qp, dp) in qb.v4l2_planes.iter_mut().zip(dq_planes.iter()) {
                        qp.bytesused = dp.bytesused;
                    }
                    unsafe { vidioc_qbuf(fd, &mut dq) }
                        .map_err(|e| format!("QBUF({index}) failed: {e}"))?;

                    let _ = response.send((format.clone(), bytes));
                }
            }
        }
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    }
    .await;

    let mut stream_type = buf_type;
    let _ = unsafe { vidioc_streamoff(fd, &mut stream_type) };
    for buf in &buffers {
        for plane in &buf.planes {
            unsafe {
                libc::munmap(plane.ptr, plane.len);
            }
        }
    }
    let mut req = V4l2RequestBuffers {
        count: 0,
        type_: buf_type,
        memory: mem_type,
        capabilities: 0,
        flags: 0,
        reserved: [0; 3],
    };
    let _ = unsafe { vidioc_reqbufs(fd, &mut req) };

    result
}
