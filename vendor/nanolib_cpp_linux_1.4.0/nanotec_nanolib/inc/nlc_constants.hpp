#pragma once

namespace nlc {

constexpr int NANOTEC_VENDOR_ID = 0x26C;

enum class DataTransferInfo {
		FileOpen,
		Init,
		Progress,
		Reboot,
		Finished
};

enum class BusScanInfo {
	Start,
	Progress,
	FoundDevice,
	Finished
};

enum class DeviceConnectionStateInfo {
	Disconnected,
	Connected,
	ConnectedBootloader
};

/**
* @brief String values
*/

struct CanOpenNmtState {
	const std::string STOPPED = "STOPPED";
	const std::string PRE_OPERATIONAL = "PRE_OPERATIONAL";
	const std::string OPERATIONAL = "OPERATIONAL";
	const std::string INITIALIZATION = "INITIALIZATION";
	const std::string UNKNOWN = "UNKNOWN";
};

/**
* @brief String values
*/
struct CanOpenNmtService { 
	const std::string START = "START";
	const std::string STOP = "STOP";
	const std::string PRE_OPERATIONAL = "PRE_OPERATIONAL";
	const std::string RESET = "RESET";
	const std::string RESET_COMMUNICATION = "RESET_COMMUNICATION";
};

/**
 * EtherCAT slave/network states
 */
struct EtherCATState {
	const std::string NONE = "NONE";
	const std::string INIT = "INIT";
	const std::string PRE_OPERATIONAL = "PRE-OPERATIONAL";
	const std::string BOOT = "BOOT";
	const std::string SAFE_OPERATIONAL = "SAFE-OPERATIONAL";
	const std::string OPERATIONAL = "OPERATIONAL";
};

const struct CanOpenNmtState canOpenNmtState;
const struct CanOpenNmtService canOpenNmtService;
const struct EtherCATState EtherCATStates;

}