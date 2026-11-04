from conan import ConanFile
from conan.tools.build import can_run
from conan.tools.cmake import CMake, cmake_layout
import os


class IronpagesTestPackage(ConanFile):
    settings = "os", "arch", "compiler", "build_type"
    generators = "CMakeDeps", "CMakeToolchain"

    def layout(self):
        cmake_layout(self)

    def requirements(self):
        self.requires(self.tested_reference_str)

    def build(self):
        cmake = CMake(self)
        cmake.configure()
        cmake.build()

    def test(self):
        if can_run(self):
            for executable in ("ironpages_c_consumer", "ironpages_cpp_consumer"):
                path = os.path.join(self.cpp.build.bindir, executable)
                self.run(path, env="conanrun")
