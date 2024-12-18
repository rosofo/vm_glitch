bundle:
    cargo xtask bundle-universal vm_glitch --release
    just install

install:
    rm -rf ~/Library/Audio/Plug-Ins/VST3/VM\ Glitch.vst3
    cp -r target/bundled/VM\ Glitch.vst3 ~/Library/Audio/Plug-Ins/VST3/