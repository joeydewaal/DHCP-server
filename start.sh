cargo build
sudo -S setcap CAP_NET_BIND_SERVICE=+eip /home/joey/dev/DHCP-server/target/debug/dhcp
cargo run
