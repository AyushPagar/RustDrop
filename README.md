# RustDrop

RustDrop shares files between devices on the same local network.

## Run on Windows

Open PowerShell in this project folder and run:

```powershell
cargo run
```

Keep the terminal open while sharing. On the computer running RustDrop, use `ipconfig` and find the Wi-Fi adapter's IPv4 Address. On another device connected to the same Wi-Fi, open `http://<IPv4-address>:3000` (for example, `http://192.168.0.107:3000`).

If another device cannot connect, allow RustDrop through Windows Firewall on private networks. Uploaded files are stored in the `uploads` folder by default. Set `RUSTDROP_DATA_DIR` to use a different folder.
