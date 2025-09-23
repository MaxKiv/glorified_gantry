#pragma once
#include "device_id.hpp"
#include "result.hpp"
#include "nlc_constants.hpp"

#include <optional>
#include <string>

namespace nlc {

/**
	* @brief Parent class for callbacks
	* 
 */
class NlcCallback {
public:
	virtual ~NlcCallback() {
	}
	virtual ResultVoid callback() = 0;
};

/**
	* @brief Callback class used in data transfers (firmware update, NanoJ upload)
 *
 * Usage:
 *
 * -# Define a class that extends this class with a custom callback method implementation
 * -# Use the instances of the new class in NanoLibAccessor.uploadFirmware(...) calls
 * 
 */
class NlcDataTransferCallback {
public:
	virtual ~NlcDataTransferCallback() {
	}
	virtual ResultVoid callback(nlc::DataTransferInfo info,
									  int32_t data)
		= 0;
};

/**
	* @brief Callback class used in bus scanning
	* 
	* Usage:
	* 
	* -# Define a class that extends this class with a custom callback method implementation
	* -# Use the instances of the new class in NanoLibAccessor.scanDevices(...) calls
	* 
*/
class NlcScanBusCallback {
public:
	virtual ~NlcScanBusCallback() {
	}
	virtual ResultVoid callback(nlc::BusScanInfo info, std::vector<DeviceId> const &devicesFound,
								int32_t data)
		= 0;
};
							
/**
	* @brief Callback class used for logging callbacks
	* 
	* Usage:
	* 
	* -# Define a class that extends this class with a custom callback method implementation
	* -# Use pointer to the instances of the new class to set callback with 
	*    NanoLibAccessor->setLoggingCallback(...)
	* 
*/
class NlcLoggingCallback {
public:
	virtual ~NlcLoggingCallback() {
	}
	virtual void callback(const std::string &payload_str, 
                            const std::string &formatted_str,
                            const std::string &logger_name, 
                            const unsigned int log_level, 
                            const std::uint64_t time_since_epoch,
                            const size_t thread_id)
		= 0;
};

}