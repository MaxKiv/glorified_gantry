#pragma once

#include <cstdint>
#include <iomanip> // for setfill and setw
#include <sstream>
#include <string>

namespace nlc {

/**
	* @brief A Class representing a handle for controlling a device on a bus
	* 
*/
struct DeviceHandle {
#ifndef SWIG_CSHARP
	DeviceHandle() : handle(0) {
	}

	DeviceHandle(uint32_t handle) : handle(handle) {
	}

	bool equals(DeviceHandle const other) const {
		return handle == other.handle;
	}

	std::string toString() const {
		std::stringstream stream;
		stream << "0x";
		stream << std::setfill('0') << std::setw(4) << std::hex << std::uppercase;
		stream << handle;
		return stream.str();
	}

	uint32_t get() const {
		return handle;
	}

private:
#endif
	uint32_t handle;
};

} // namespace nlc
