add_rules("plugin.compile_commands.autoupdate", {outputdir = "."})
add_requires("raylib")

target("pixeltown")
    set_kind("binary")
    set_targetdir("build")
    add_includedirs("src")
    add_files("src/*.c")

    add_rules("utils.bin2c", {extensions = {".ttf"}})
    add_files("assets/fonts/*.ttf")

    add_packages("raylib")
