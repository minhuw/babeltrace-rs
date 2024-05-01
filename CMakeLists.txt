# Corrosion needs CMake > 3.15
cmake_minimum_required(VERSION 3.15)

project(babeltrace_plugins LANGUAGES C)

option(NIX_BUILD "build for nix" $ENV{NIX_BUILD})

find_package(PkgConfig REQUIRED) 

pkg_check_modules(babeltrace2 REQUIRED IMPORTED_TARGET babeltrace2)

if (NOT NIX_BUILD)
    # Fetch Corrosion
    include(FetchContent)

    FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
        GIT_TAG v0.4 
    )
    FetchContent_MakeAvailable(Corrosion)

    corrosion_import_crate(MANIFEST_PATH Cargo.toml)
endif()

add_library(plugins SHARED module.c)

target_link_libraries(plugins PUBLIC db_converter babeltrace2)

install(TARGETS plugins DESTINATION lib/babeltrace2/plugins)
