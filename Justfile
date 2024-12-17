bundle:
    cargo xtask bundle vm_glitch --release
    rm -rf ~/Library/Audio/Plug-Ins/VST3/VM\ Glitch.vst3
    cp -r target/bundled/VM\ Glitch.vst3 ~/Library/Audio/Plug-Ins/VST3/