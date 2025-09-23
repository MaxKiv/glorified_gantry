#pragma once
#include <string>

namespace nlc {

/**
 * @brief Identifies a bus hardware one-to-one and onto
 *
 * To differentiate different bus hardware, this id class is used. It holds information about
 * -# the hardware itself (e.g. adapter name, network adapter, ...)
 * -# the protocol to use (e.g. MODBUS TCP or EtherCAT)
 * -# the bus hardware specifier (e.g. serial port name or MAC address)
 * -# a friendly name
 *
 * The class has no setter functions to be immutable from point of creation on.
 */
class BusHardwareId {
public:
	/**
		* @brief Construct a new Bus Hardware Id object
		*
		* @param busHardware_ The hardware type (ZK-USB-CAN-1 for example)
		* @param protocol_ Bus communication protocol (CANopen for example)
		* @param hardwareSpecifier_  The specifier of the hardware (COM3 for example)
		* @param name_  A friendly name (for example "AdapterName (Port)" )
	*/
	BusHardwareId(std::string const &busHardware_, std::string const &protocol_,
				  std::string const &hardwareSpecifier_, std::string const &name_)
		: busHardware(busHardware_), protocol(protocol_), hardwareSpecifier(hardwareSpecifier_),
		name(name_) {
	}

	/**
	 * @brief Construct a new Bus Hardware Id object
	 *
	 * @param busHardware_ The hardware type (ZK-USB-CAN-1 for example)
	 * @param protocol_ Bus communication protocol (CANopen for example)
	 * @param hardwareSpecifier_  The specifier of the hardware (COM3 for example)
	 * @param extraHardwareSpecifier_  The extra specifier of the hardware (USB location info for example)
	 * @param name_  A friendly name (for example "AdapterName (Port)" )
	 */
	BusHardwareId(std::string const &busHardware_, std::string const &protocol_,
				  std::string const &hardwareSpecifier_,
				  std::string const &extraHardwareSpecifier_, std::string const &name_)
		: busHardware(busHardware_), protocol(protocol_), hardwareSpecifier(hardwareSpecifier_),
		  extraHardwareSpecifier(extraHardwareSpecifier_),
		  name(name_) {
	}

	virtual ~BusHardwareId() = default;

	/**
	 * @brief Copy constructor
	 * 
	 * @param busHardwareId BusHardwareId to copy from
	 */
	BusHardwareId(BusHardwareId const &) = default;

	/**
	 * @brief Construct a invalid BusHardwareId Id object
	 */
	BusHardwareId() = default;

	/**
	 * @brief Get the bus hardware as a string representation
	 *
	 * @return std::string
	 */
	std::string getBusHardware() const {
		return busHardware;
	}

	/**
		* @brief Get the bus protocol as a string representation
		*
		* @return std::string
	*/
	std::string getProtocol() const {
		return protocol;
	}

	/**
	 * @brief Get the bus hardware specifier (e.g. network name) as a string representation
	 *
	 * @return std::string
	 */
	std::string getHardwareSpecifier() const {
		return hardwareSpecifier;
	}

	/**
	 * @brief Get the bus extra hardware specifier (e.g. MAC address) as a string representation
	 *
	 * @return std::string
	 */
	std::string getExtraHardwareSpecifier() const {
		return extraHardwareSpecifier;
	}

	/**
	 * @brief Get the bus hardware friendly name
	 *
	 * @return std::string
	 */
	std::string getName() const {
		return name;
	}


	/**
	 * @brief Compares itself to given BusHardwareId
	 *
	 * @param other
	 * @return true if both are equal in all values
	 * @return false if the values differ
	 */
	bool equals(BusHardwareId const &other) const {
		return (busHardware.compare(other.busHardware) == 0 && protocol.compare(other.protocol) == 0
				&& hardwareSpecifier.compare(other.hardwareSpecifier) == 0
				&& extraHardwareSpecifier.compare(other.extraHardwareSpecifier) == 0);
	}

	/**
	 * @brief Returns a string representation of the bus hardware id
	 *
	 * @return std::string
	 */
	std::string toString() const {
		std::string result = "BusHardwareId(" + name + ") [bus hardware: " + busHardware + ", protocol: " + protocol
			   + ", hardware specifier: " + hardwareSpecifier;
		if (!extraHardwareSpecifier.empty()) {
			result += ", extra hardware specifier: " + extraHardwareSpecifier;
		}
		result += "]";
		return result;
	}

private:
	std::string busHardware;
	std::string protocol;
	std::string hardwareSpecifier;
	std::string extraHardwareSpecifier;
	std::string name;
};

} // namespace nlc
