bundle *CARGO_ARGS:
    @echo "Pass `-F tracing` to bundle with tracing enabled"
    cargo xtask bundle-universal vm_glitch --release {{CARGO_ARGS}}
    just install

install:
    rm -rf ~/Library/Audio/Plug-Ins/VST3/VM\ Glitch.vst3
    cp -r target/bundled/VM\ Glitch.vst3 ~/Library/Audio/Plug-Ins/VST3/

trace:
    @echo "Starting the profiler frontend."
    @echo "Make sure you bundle with `just bundle -F tracing`, then connect the frontend to localhost."
    tracy || (brew install tracy && tracy)