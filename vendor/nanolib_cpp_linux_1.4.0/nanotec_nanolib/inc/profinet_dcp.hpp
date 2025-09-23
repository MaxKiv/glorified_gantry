#pragma once

#include "bus_hardware_id.hpp"
#include "result.hpp"

#include <array>

namespace nlc {

/**
 * @brief Profinet device data
 */
struct ProfinetDevice {
	std::string deviceName;
	std::string deviceVendor;
	std::array<uint8_t, 6> macAddress;
	uint32_t ipAddress;		 // big endian - 0xC0A80002 for 192.168.0.2
	uint32_t netMask;		 // big endian - 0xFFFF0000 for 255.255.0.0
	uint32_t defaultGateway; // big endian - 0xC0A80001 for 192.168.0.1

	ProfinetDevice() {

		macAddress = {0, 0, 0, 0, 0, 0};
		ipAddress = 0;
		netMask = 0;
		defaultGateway = 0;
	}
};

class ResultProfinetDevices : public Result {
public:
	ResultProfinetDevices(const std::vector<ProfinetDevice> &profinetDevices)
		: result(profinetDevices) {
	}

	ResultProfinetDevices(const Result& result) 
		: Result(result) {
	}

	ResultProfinetDevices(const std::string &errorText,
						  NlcErrorCode errorCode = NlcErrorCode::GeneralError,
						  uint32_t extendedErrorCode = 0)
		: Result(errorCode, extendedErrorCode, errorText) {
	}

public:

	const std::vector<ProfinetDevice> &getResult() const {
		return result;
	}

private:
	std::vector<ProfinetDevice> result;
};

/**
 * @brief Profinet DCP interface
 * 
 * Under Windows, the implementation of the interface uses:
 * Npcap (https://npcap.com) Tested with versions 1.60 & 1.30
 * or
 * WinPcap (https://www.winpcap.org/) Tested with last version 4.1.3
 *
 * This means dynamically loading the wpcap.dll library,
 * which is searched in the following order:
 * - nanolib.dll directory
 * - Windows system directory - %SystemRoot%\System32
 * - Npcap installation directory - %SystemRoot%\System32\Npcap
 * - environment path
 *
 * At this stage Win10Pcap (https://www.win10pcap.org) is NOT SUPPORTED.
 * 
 * 
 * Under Linux, the calling application must have 
 * CAP_NET_ADMIN and CAP_NET_RAW capabilities.
 * To enable: 
 * sudo setcap 'cap_net_admin,cap_net_raw+eip' ./executable
 */
class ProfinetDCP {
public:

	virtual ~ProfinetDCP() = 0;

	/**
	 * @brief Devices scan timeout
	 * 
	 * The default value is 2000 msec
	 */
	virtual uint32_t getScanTimeout() const = 0;
	virtual void setScanTimeout(uint32_t timeoutMsec) = 0;

	/**
	 * @brief Response timeout for setup, reset and blink operations
	 * 
	 * The default value is 1000 msec
	 */
	virtual uint32_t getResponseTimeout() const = 0;
	virtual void setResponseTimeout(uint32_t timeoutMsec) = 0;

	/**
	 *  @brief Checks the availability of the Profinet DCP service
	 * 
	 *  - validity and availability of the network adapter
	 *  - Windows: WinPcap / Npcap availability, 
	 *    Linux: CAP_NET_ADMIN and CAP_NET_RAW capabilities
	 */
	virtual ResultVoid isServiceAvailable(const BusHardwareId &busHardwareId) = 0;

	/**
	 * @brief Scanning the hardware bus for the presence of Profinet devices
	 */
	virtual ResultProfinetDevices scanProfinetDevices(const BusHardwareId &busHardwareId) = 0;


	/**
	 * @brief Establishing the following device settings:
	 * - device name
	 * - IP address
	 * - network mask
	 * - default gateway
	 */
	virtual ResultVoid setupProfinetDevice(const BusHardwareId &busHardwareId,
							   const ProfinetDevice &profinetDevice, bool savePermanent)
		= 0;

	/**
	 * @brief Stops the device and reset to factory defaults
	 */
	virtual ResultVoid resetProfinetDevice(const BusHardwareId &busHardwareId,
										   const ProfinetDevice &profinetDevice)
		= 0;

	/**
	 * @brief Blink the device
	 */
	virtual ResultVoid blinkProfinetDevice(const BusHardwareId &busHardwareId,
							   const ProfinetDevice &profinetDevice)
		= 0;

	/**
	 * @brief Checks the IP address of the device. 
	 */
	virtual ResultVoid validateProfinetDeviceIp(const BusHardwareId &busHardwareId,
												const ProfinetDevice &profinetDevice)
		= 0;

};

inline ProfinetDCP::~ProfinetDCP() {
}

} // namespace nlc
