call F:\env.bat

set

cmake -G "Visual Studio 16 2019" -DCMAKE_TOOLCHAIN_FILE="F:\vcpkg\scripts\buildsystems\vcpkg.cmake" -G"NMake Makefiles" -DWIN32_WIN64_CROSS_COMPILE=1 -DNOSERVER=1 -DCMAKE_INSTALL_PREFIX=F:\dist -DCMAKE_BUILD_TYPE="Release" -DSDL2_BUILDING_LIBRARY=1
nmake install
copy F:\vcpkg\installed\x64-windows\bin\ogg.dll F:\dist
copy F:\vcpkg\installed\x64-windows\bin\vorbis.dll F:\dist
copy F:\vcpkg\installed\x64-windows\bin\vorbisfile.dll F:\dist
powershell Compress-Archive . dist.zip