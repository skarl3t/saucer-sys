cmake_minimum_required(VERSION 3.21)
project(saucer-bindings LANGUAGES CXX VERSION 8.0.0)

# +-------------------------------------------------------------------------------------------------------+
# | Library switches                                                                                      |
# +-------------------------------------------------------------------------------------------------------+

option(saucer_desktop "Enable support for the desktop module" ON)
option(saucer_pdf "Enable support for the pdf module" ON)
option(saucer_loop "Enable support for the loop module" OFF)

# Rust relies heavily on static linking...
option(saucers_static "Build a static library" ON)

# +-------------------------------------------------------------------------------------------------------+
# | CMake options                                                                                         |
# +-------------------------------------------------------------------------------------------------------+

set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# +-------------------------------------------------------------------------------------------------------+
# | Setup library                                                                                         |
# +-------------------------------------------------------------------------------------------------------+

if (saucers_static)
    add_library(${PROJECT_NAME} STATIC)
else ()
    add_library(${PROJECT_NAME} SHARED)
endif ()


add_library(saucer::bindings ALIAS ${PROJECT_NAME})

target_compile_features(${PROJECT_NAME} PRIVATE cxx_std_23)
set_target_properties(${PROJECT_NAME} PROPERTIES CXX_STANDARD 23 CXX_EXTENSIONS OFF CXX_STANDARD_REQUIRED ON)

if (PROJECT_IS_TOP_LEVEL AND NOT MSVC AND NOT CMAKE_CXX_SIMULATE_ID MATCHES "MSVC")
    target_compile_options(${PROJECT_NAME} PRIVATE -Wall -Wextra -Wpedantic -Werror -pedantic -pedantic-errors -Wfatal-errors)
endif ()

if (NOT MSVC)
    target_compile_options(${PROJECT_NAME} PRIVATE -Wno-unknown-warning-option -Wno-missing-field-initializers -Wno-cast-function-type)
endif ()

# +-------------------------------------------------------------------------------------------------------+
# | Export header                                                                                         |
# +-------------------------------------------------------------------------------------------------------+

include("cmake/hide.cmake")
include("cmake/export.cmake")

saucer_bindings_hide_symbols(${PROJECT_NAME})
saucer_bindings_export(${PROJECT_NAME} "SAUCER_EXPORT")

# +-------------------------------------------------------------------------------------------------------+
# | Setup version header                                                                                  |
# +-------------------------------------------------------------------------------------------------------+

configure_file("template/version.hpp.in" "${CMAKE_CURRENT_SOURCE_DIR}/private/version.hpp")

# +-------------------------------------------------------------------------------------------------------+
# | Include directories                                                                                   |
# +-------------------------------------------------------------------------------------------------------+

target_include_directories(${PROJECT_NAME} PUBLIC "include")
target_include_directories(${PROJECT_NAME} PRIVATE "include/saucer" "private")

# +-------------------------------------------------------------------------------------------------------+
# | Add Sources                                                                                           |
# +-------------------------------------------------------------------------------------------------------+

target_sources(${PROJECT_NAME} PRIVATE
        "src/app.cpp"
        "src/window.cpp"
        "src/webview.cpp"

        "src/url.cpp"
        "src/icon.cpp"
        "src/stash.cpp"

        "src/scheme.cpp"
        "src/permission.cpp"
        "src/navigation.cpp"
)

# +-------------------------------------------------------------------------------------------------------+
# | Setup Dependencies                                                                                    |
# +-------------------------------------------------------------------------------------------------------+

# Looks like there's a bug with local packages that causes functions like `saucer_add_module` to be missing
set(CPM_DOWNLOAD_ALL ON)

include("cmake/cpm.cmake")

CPMFindPackage(
        NAME saucer
        VERSION 8.0.0
        GIT_REPOSITORY "https://github.com/saucer/saucer"
        OPTIONS
        "saucer_static ON"
        "CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS ON"
)

target_link_libraries(${PROJECT_NAME} PUBLIC saucer::saucer saucer::private)

# +-------------------------------------------------------------------------------------------------------+
# | Setup Natives                                                                                         |
# +-------------------------------------------------------------------------------------------------------+

# Seems to break macOS builds, just disable them since we're not using native anyway
# if (saucer_backend STREQUAL "WebKit")
#     enable_language(OBJCXX)
#     configure_file("src/native.cpp" "${CMAKE_CURRENT_SOURCE_DIR}/src/native.mm" COPYONLY)
#     target_sources(${PROJECT_NAME} PRIVATE "src/native.mm")
# else ()
#     target_sources(${PROJECT_NAME} PRIVATE "src/native.cpp")
# endif ()

# +-------------------------------------------------------------------------------------------------------+
# | Setup Modules                                                                                         |
# +-------------------------------------------------------------------------------------------------------+

include("cmake/module.cmake")

if (saucer_desktop)
    add_subdirectory("modules/desktop")
endif ()

if (saucer_pdf)
    add_subdirectory("modules/pdf")
endif ()

if (saucer_loop)
    add_subdirectory("modules/loop")
endif ()

install(
        TARGETS ${PROJECT_NAME} saucer saucer-desktop saucer-pdf saucer-bindings-desktop saucer-bindings-pdf coco
        ARCHIVE DESTINATION "."
        LIBRARY DESTINATION "."
        RUNTIME DESTINATION "."
)