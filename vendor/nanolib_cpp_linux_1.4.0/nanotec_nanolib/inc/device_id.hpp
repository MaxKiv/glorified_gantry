#pragma once

#include <string>
#include <iomanip>
#include <sstream>
#include <vector>
#include <cstdint>

#include "bus_hardware_id.hpp"

namespace nlc {

/**
 * @brief This class indentifies a device on the bus.
 *
 * To differentiate devices on bus this class holds information about
 * -# the identifier of the hardware adapter
 * -# a device identifier
 * -# a description
 *
 * The meaning of the device id and the description values are depending on the bus used. For
 * instance a can bus may use the integer id.
 *
 * The class has no setter functions to be immutable from point of creation on.
 */

class DeviceId {
public:
	/**
	 * @brief Construct a new Device Id object
	 *
	 * @param busHardwareId The identifier of the bus
	 * @param deviceId_ An index, meaning is depending on the bus (e.g. CANOpen node id).
	 * @param description_ A description (may be empty), meaning is depending on the bus.
	 */
	DeviceId(BusHardwareId const& busHardwareId_, unsigned int deviceId_, std::string const &description_)
		: busHardwareId(busHardwareId_), deviceId(deviceId_), description(description_) {
	}

	/**
	 * @brief Construct a new Device Id object
	 *
	 * @param busHardwareId The identifier of the bus
	 * @param deviceId_ An index, meaning is depending on the bus (e.g. CANOpen node id).
	 * @param description_ A description (may be empty), meaning is depending on the bus.
	 * @param extraId_ A additional Id (may be empty), meaning is depending on the bus.
	 * @param extraStringId_ A additional String Id (may be empty), meaning is depending on the bus.
	 */
	DeviceId(BusHardwareId const &busHardwareId_, unsigned int deviceId_,
			 std::string const &description_, std::vector<uint8_t> const &extraId_,
			 std::string const &extraStringId_)
		: busHardwareId(busHardwareId_), deviceId(deviceId_), description(description_),
		  extraId(extraId_), extraStringId(extraStringId_) {
	}

	/**
	 * @brief Copy constructor
	 * 
	 * @param deviceId DeviceId to copy from
	 */
	DeviceId(DeviceId const &) = default;

	/**
	 * @brief Construct an invalid Device Id object
	 */
	DeviceId() {
	}

	/**
	* @brief Get the bus hardware id
	*
	* @return BusHardwareId
	*/
	BusHardwareId getBusHardwareId() const {
		return busHardwareId;
	}

	/**
	 * @brief Get the device id (may be unused)
	 *
	 * @return unsigned int
	 */
	unsigned int getDeviceId() const {
		return deviceId;
	}

	/**
	 * @brief Get the description of the device (may be unused)
	 *
	 * @return std::string
	 */
	std::string getDescription() const {
		return description;
	}

	/**
	 * @brief Get the extra ID of the device (may be unused)
	 *
	 * @return const std::vector<uint8_t> &
	 */
	const std::vector<uint8_t> &getExtraId() const {
		return extraId;
	}

	/**
	 * @brief Get the extra string ID of the device (may be unused)
	 *
	 * @return std::string
	 */
	std::string getExtraStringId() const {
		return extraStringId;
	}

	/**
	 * @brief Returns a string representation of the object
	 *
	 * @return std::string
	 */
	std::string toString() const {

		std::string result = "DeviceId [" + busHardwareId.toString() + ", device id: "
							 + std::to_string(deviceId) + ", description: " + description;
		
		if (!extraId.empty()) {
			std::stringstream ss;
			ss << ", extra id: " << std::hex;
			for (size_t i = 0; i < extraId.size(); ++i) {
				ss << std::setw(2) << std::setfill('0') << (int)extraId[i];
			}

			result += ss.str();			
		}

		if (!extraStringId.empty()) {
			result += ", extra string id: " + extraStringId;
		}

		result += "]";

		return result;
	}

	/**
	 * @brief Compares itself to another object
	 *
	 * @return boolean
	 */
	bool equals(DeviceId const &other) const {
		return (busHardwareId.equals(other.busHardwareId) && deviceId == (other.deviceId)
				&& extraId == other.getExtraId() && extraStringId == other.getExtraStringId()
			/*&& description.compare(other.description) == 0*/);
	}
	
protected:
	BusHardwareId busHardwareId{"", "", "", ""};
	unsigned int deviceId{0};
	std::string description{""};
	std::vector<uint8_t> extraId;
	std::string extraStringId;
};

} // namespace nlc
