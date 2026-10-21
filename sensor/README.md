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
storage send target/thumbv7em-none-eabihf/release/sensor.fap /{Location of flipperzero}/apps/Examples/sensor.fap
'''