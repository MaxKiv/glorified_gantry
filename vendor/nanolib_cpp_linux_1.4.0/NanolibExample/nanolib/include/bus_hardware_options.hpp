#pragma once

#include <map>
#include <string>

namespace nlc {
/**
 * @brief This class holds all options necessary to open a bus hardware.
 * 
 * The options are hold in a key-value list of strings.
 */
class BusHardwareOptions {
public:
	/**
	 * @brief Construct a new bus hardware option object.
	 *
	 * Use the function addOption(std::string const &key, std::string const &value)
	 * to add key-value pairs.
	 * 
	 */
	BusHardwareOptions() = default;

	/**
	 * @brief Construct a new Bus Hardware Options object with the key-value map already in place.
	 *
	 * @param options A map with options for the bus hardware to operate.
	 */
	explicit BusHardwareOptions(std::map<std::string, std::string> const &options) : options(options) {
	}

	/**
	 * @brief Adds given key and value to itself
	 *
	 * @param key (e.g. nlc.CanBus().BAUD_RATE_OPTIONS_NAME, see bus_hw_options_defaults.hpp)
	 * @param value (e.g. nlc.CanBus().baudRate.BAUD_RATE_1000K, see bus_hw_options_defaults.hpp)
	 */
	void addOption(std::string const &key, std::string const &value) {
		options.insert(std::make_pair(key, value));
	}

	/**
	 * @brief Get all of the added key-value pairs
	 *
	 * @return std::map<std::string, std::string>
	 */
	std::map<std::string, std::string> getOptions() const {
		return options;
	}

	/**
	 * @brief Compares itself to the given BusHardwareOptions
	 *
	 * @param other Another object of the same class
	 * 
	 * @return true in case the other object has all of the exact same options
	 * @return false in case the other object has different keys or values
	 */
	bool equals(BusHardwareOptions const &other) const {
		if (options.size() != other.options.size()) {
			return false;
		}

		std::map<std::string, std::string>::const_iterator it;
		for (it = options.begin(); it != options.end(); it++) {
			std::string key = it->first;
			// check, if key exists
			if (other.options.find(key) == other.options.end()) {
				return false;
			}
			// ok, key exists, check for value
			std::string value = it->second;
			if (other.options.at(key).compare(value) != 0) {
				return false;
			}
		}

		return true;
	}

	/**
	 * @brief Returns a string representation of all keys and corresponding values
	 *
	 * @return std::string
	 */
	std::string toString() const {
		std::string retString("BusHardwareOptions [options: ");

		std::map<std::string, std::string>::const_iterator it;
		for (it = options.begin(); it != options.end(); it++) {
			retString += it->first + ":" + it->second + " ";
		}

		retString += "]";

		return retString;
	}

private:
	std::map<std::string, std::string> options;
};

} // namespace nlc
