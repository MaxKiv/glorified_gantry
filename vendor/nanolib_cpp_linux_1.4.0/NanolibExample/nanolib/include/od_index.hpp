#pragma once

#include <cstdint>
#include <iomanip> // for setfill and setw
#include <sstream>
#include <string>

namespace nlc {

/**
 * @brief This class is a wrapper for an object dictionary index and subindex
 *
 * The object dictionary of a device is build up in up to 65535 (0xFFFF) rows and 255
 * (0xFF) columns. The rows are not continuous, there are gaps in between. Look in the
 * CANopen standard for further details.
 *
 * This class is immutable from the creation up.
 */
struct OdIndex {
#ifndef SWIG_CSHARP
	/**
	 * @brief Construct a new OdIndex object
	 *
	 * @param index From 0 to 65535 (0xFFFF) (incl.)
	 * @param subIndex From 0 to 255 (0xFF) (incl.)
	 */
	OdIndex(uint16_t index, uint8_t subIndex) : index(index), subIndex(subIndex) {
	}

	OdIndex() : index(0), subIndex(0) {
	}

	/**
	 * @brief Get the index (from 0x0000 to 0xFFFF)
	 *
	 * @return uint16_t
	 */
	uint16_t getIndex() const {
		return index;
	}

	/**
	 * @brief Get the sub-index (from 0x00 to 0xFF)
	 *
	 * @return uint8_t
	 */
	uint8_t getSubIndex() const {
		return subIndex;
	}

	/**
	 * @brief Returns string representation of the od index/sub-index
	 *
	 * The writing within Nanotec of ods is
	 *
	 * 0xIIII:0xSS
	 *
	 * where
	 * - I is the index from 0x0000 to 0xFFFF
	 * - S is the sub-index from 0x00 to 0xFF
	 *
	 * @return Default string representation
	 */
	std::string toString() const {
		std::stringstream stream;

		stream << "0x";
		// fill leading numbers with 0, the width of the number is 4, everything is an uppercase
		// hexadecimal number
		stream << std::setfill('0') << std::setw(4) << std::hex << std::uppercase;
		stream << static_cast<int>(
			index); // the uint16_t and uint8_t data types are unsafe with using streams
		stream << ":0x";
		stream << std::setw(2);
		stream << static_cast<int>(subIndex);

		return stream.str();
	}

private:
#endif
	uint16_t index;
	uint8_t subIndex;
};

} // namespace nlc
