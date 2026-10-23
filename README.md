# RustDrop

A lightweight file-sharing app built with Rust. Upload files from one device and download them on another device connected to the same Wi-Fi network.

## Features

- Upload files through a web dashboard
- Drag and drop files to upload
- Browse and download shared files
- Use the dashboard on computers and phones

## Run locally

You’ll need [Rust](https://www.rust-lang.org/tools/install).

In PowerShell, go to the project folder and run:

```powershell
cargo run

Keep the terminal open while sharing. On the computer running RustDrop, use `ipconfig` and find the Wi-Fi adapter's IPv4 Address. On another device connected to the same Wi-Fi, open `http://<IPv4-address>:3000` (for example, `http://192.168.0.107:3000`).

If another device cannot connect, allow RustDrop through Windows Firewall on private networks. Uploaded files are stored in the `uploads` folder by default. Set `RUSTDROP_DATA_DIR` to use a different folder.
