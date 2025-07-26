# diskblock

```bash
flox activate
mkdir live
cargo build --release
cp ./target/release/diskblock-rust ./live/diskblock
sudo cp daemon.plist /Library/LaunchDaemons/com.deitloff.alec.diskblock.plist
sudo chown root:wheel daemon.plist
sudo launchctl load -w /Library/LaunchDaemons/com.deitloff.alec.diskblock.plist
```
