# Backend for Industrial Computer Vision

This backend software tries to implement the most common types of communication in industrial communication and the most
common CV backends.

## Industrial communication

The two most common types of industrial communication are:

- MQTT
- OPC UA

thus, both of them are implemented.

## CV backends

The most common computer vision backend is ONNX. It is universally supported on almost any hardware platform and any
operating system. Furthermore, the Hailo NPU is supported through the hailo runtime.

## Building the project

The project should run on major operating systems (Linux, macOS, Windows). It is only officially tested using Linux. The
Hailo NPU is not supported on macOS!

The different CV backends are separated by feature flags.

To build the project, run: `cargo build --release [--features "cpu cuda rocm openvino coreml hailo"]`

Enabling more than one feature in a production build is discouraged. The default feature is `cpu`.

---

## Contributing

This project is open to contributions. Be aware of the following criteria:

- The project is used commercially by GROB-WERKE GmbH & Co. KG. They have an enterprise license and are thus exempt from
  the normal AGPLv3 license. No other enterprise licenses will be granted!
  If you don't want your code to be used commercially, please fork the project.
- LLM contributions are prohibited. Creating this project from scratch took a lot of time and patience.

Before committing and pushing to the upstream repo, be sure to execute the following steps:

1. Reformat the code using `cargo fmt`
2. Run the linter using `cargo clippy -- -D warnings`
3. Run the tests using `cargo test`

---

&copy; 2026 LOENS2 and contributors.