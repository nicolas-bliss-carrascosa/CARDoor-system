## Set up the compilation to a flipper zero

Step 1:
'''
rustup target add --toolchain nightly thumbv7em-none-eabihf
'''

## Build and copy to flipper zero
Step 1:
'''
cargo build
'''

Step 2:
'''
sudo storage send target/thumbv7em-none-eabihf/release/sensor.fap /ext/apps/Examples/sensor.fap
'''


To Build and run automatically
'''
sudo cargo watch -s 'cargo build --release && run-fap target/thumbv7em-none-eabihf/release/sensor.fap'
'''