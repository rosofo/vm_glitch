vst3_dir := if os() == "macos" {
    x"~/Library/Audio/Plug-Ins/VST3"
} else if os() == "linux" {
    x"~/.vst3"
} else {
    error("Unsupported OS: '{{os()}}'")
}

# The CoreAudio SDK is required for universal bundle building
bundle_command := if os() == "macos" { "bundle-universal" } else { "bundle" }

bundle *CARGO_ARGS: && install
    @echo "Pass '-F tracing' to bundle with tracing enabled"
    cargo xtask {{bundle_command}} vm_glitch --release {{CARGO_ARGS}}

install:
    rm -rf "{{vst3_dir}}/VM Glitch.vst3"
    cp -r "target/bundled/VM Glitch.vst3" "{{vst3_dir}}/VM Glitch.vst3"

trace:
    @echo "Starting the profiler frontend."
    @echo "Make sure you bundle with 'just bundle -F tracing', then connect the frontend to localhost."
    tracy || (brew install tracy && tracy)