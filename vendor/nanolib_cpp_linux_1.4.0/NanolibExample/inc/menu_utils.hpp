/*
* Nanotec Nanolib example
* Copyright (C) Nanotec GmbH & Co. KG - All Rights Reserved
*
* This product includes software developed by the
* Nanotec GmbH & Co. KG (http://www.nanotec.com/).
*
* The Nanolib interface headers and the examples source code provided are 
* licensed under the Creative Commons Attribution 4.0 Internaltional License. 
* To view a copy of this license, 
* visit https://creativecommons.org/licenses/by/4.0/ or send a letter to 
* Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.
*
* The parts of the library provided in binary format are licensed under 
* the Creative Commons Attribution-NoDerivatives 4.0 International License. 
* To view a copy of this license, 
* visit http://creativecommons.org/licenses/by-nd/4.0/ or send a letter to 
* Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
*
* @file   menu_utils.hpp
*
* @brief  Declarations of CLI menu specific classes
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#pragma once
#include <optional>
#include <string>
#include <cctype>
#include <iostream>
#include <limits>
#include <sstream>
#include <type_traits>
#include <chrono>
#include <vector>
#include "nlc_callback.hpp"
#include "nano_lib_accessor.hpp"
#include "accessor_factory.hpp"
#include "nano_lib_hw_strings.hpp"
#include "bus_hardware_id.hpp"
#include "nlc_log_level.hpp"
#include "device_id.hpp"
#include "device_handle.hpp"
#include "bus_hardware_options.hpp"
#include "result_od.hpp"
#include "od_types_helper.hpp"
#include "menu_color.hpp"
#include "scan_bus_callback_example.hpp"
#include "logging_callback_example.hpp"
#include "data_transfer_callback_example.hpp"

using namespace nlc;
using namespace std;
using namespace menu_color;

namespace nlc {

/// @brief Od index of ST unit position
const OdIndex odSIUnitPosition(0x60A8, 0x00);
/// @brief Od index of control word
const OdIndex odControlWord(0x6040, 0x00);
/// @brief Od index of status word
const OdIndex odStatusWord(0x6041, 0x00);
/// @brief Od index of home page string
const OdIndex odHomePage(0x6505, 0x00);
/// @brief Od index of NanoJ control
const OdIndex odNanoJControl(0x2300, 0x00);
/// @brief Od index of NanoJ status
const OdIndex odNanoJStatus(0x2301,0x00);
/// @brief Od index of NanoJ error
const OdIndex odNanoJError(0x2302, 0x00);
/// @brief Od index of mode of operation
const OdIndex odModeOfOperation(0x6060, 0x00);
/// @brief Od index of target velocity
const OdIndex odTargetVelocity(0x60FF, 0x00);
/// @brief Od index of profile velocity
const OdIndex odProfileVelocity(0x6081, 0x00);
/// @brief Od index of target position
const OdIndex odTargetPosition(0x607A, 0x00);
/// @brief index of Pre-defined error field
const uint16_t odErrorStackIndex = 0x1003;
/// @brief Od index of error count
const OdIndex odErrorCount(0x1003, 0x00);
/// @brief position encoder resolution - encoder increments interface #1
const OdIndex odPosEncoderIncrementsInterface1(0x60E6, 0x1);
/// @brief position encoder resolution - encoder increments interface #2
const OdIndex odPosEncoderIncrementsInterface2(0x60E6, 0x2);
/// @brief position encoder resolution - encoder increments interface #3
const OdIndex odPosEncoderIncrementsInterface3(0x60E6, 0x3);
/// @brief Motor drive submode select
const OdIndex odMotorDriveSubmodeSelect(0x3202, 0x00);
/// @brief Save all parameters to non-volatile memory
const OdIndex odStoreAllParams(0x1010, 0x01);
/// @brief Restore all default parameters
const OdIndex odRestoreAllDefParams(0x1011, 0x01);
/// @brief Restore tuning default parameters
const OdIndex odRestoreTuningDefParams(0x1011, 0x06);
/// @brief Modes of operation display
const OdIndex odModeOfOperationDisplay(0x6061, 0x00);

// bus hardware menu texts
const string BUS_HARDWARE_MENU = "Bus Hardware Menu";
const string BUS_HARDWARE_OPEN_MI = "Open Bus Hardware";
const string BUS_HARDWARE_CLOSE_MI = "Close bus hardware";
const string BUS_HARDWARE_SCAN_MI = "Scan for Bus hardware";
const string BUS_HARDWARE_CLOSE_ALL_MI = "Close all bus hardware";

// device menu texts
const string DEVICE_MENU = "Device Menu";
const string DEVICE_SCAN_MI = "Scan for Devices";
const string DEVICE_CONNECT_MENU = "Connect to device Menu";
const string DEVICE_DISCONNECT_MENU = "Disconnect from device Menu";
const string DEVICE_SELECT_ACTIVE_MENU = "Select active device";
const string DEVICE_REBOOT_MI = "Reboot device";
const string DEVICE_UPDATE_FW_MI = "Update firmware";
const string DEVICE_UPDATE_BL_MI = "Update bootloader";
const string DEVICE_UPLOAD_NANOJ_MI = "Upload NanoJ program";
const string DEVICE_RUN_NANOJ_MI = "Run NanoJ program";
const string DEVICE_STOP_NANOJ_MI = "Stop NanoJ program";

// device information menu texts
const string DEVICE_INFORMATION_MENU = "Device information Menu";
const string DEVICE_GET_VENDOR_ID_MI = "Read vendor Id";
const string DEVICE_GET_PRODUCT_CODE_MI = "Read product code";
const string DEVICE_GET_DEVICE_NAME_MI = "Read device name";
const string DEVICE_GET_HW_VERSION_MI = "Read device hardware version";
const string DEVICE_GET_FW_BUILD_ID_MI = "Read device firmware build id";
const string DEVICE_GET_BL_BUILD_ID_MI = "Read device bootloader build id";
const string DEVICE_GET_SERIAL_NUMBER_MI = "Read device serial number";
const string DEVICE_GET_UNIQUE_ID_MI = "Read device unique id";
const string DEVICE_GET_BL_VERSION_MI = "Read device bootloader version";
const string DEVICE_GET_HW_GROUP_MI = "Read device hardware group";
const string DEVICE_GET_CON_STATE_MI = "Read device connection state";
const string DEVICE_GET_ERROR_FIELD_MI = "Read device error field";
const string DEVICE_RESTORE_ALL_DEFAULT_PARAMS_MI = "Restore all default parameters";

// od interface menu texts
const string OD_INTERFACE_MENU = "Object Dictionary Interface Menu";
const string OD_ASSIGN_OD_MI = "Assign an object dictionary to active device (e.g. od.xml)";
const string OD_READ_NUMBER_MI = "readNumber (raw, untyped)";
const string OD_READ_STRING_MI = "readString";
const string OD_READ_BYTES_MI = "readBytes (raw, untyped)";
const string OD_WRITE_NUMBER_MI = "writeNumber (data bitlength needed)";
const string OD_READ_NUMBER_VIA_OD_MI = "readNumber (via OD interface, get type information)";
const string OD_WRITE_NUMBER_VIA_OD_MI = "writeNumber (via OD interface, no data bitlength needed)";

// logging menu texts
const string LOGGING_MENU = "Logging Menu";
const string LOGGING_SET_LOG_LEVEL_MI = "Set log level";
const string LOGGING_SET_LOG_CALLBACK_MI = "Set logging callback";

// log level menu texts
const string LOG_LEVEL_MENU = "Log level Menu";
const string LOG_LEVEL_TRACE_MI = "Set log level to 'Trace'";
const string LOG_LEVEL_DEBUG_MI = "Set log level to 'Debug'";
const string LOG_LEVEL_INFO_MI = "Set log level to 'Info'";
const string LOG_LEVEL_WARN_MI = "Set log level to 'Warning'";
const string LOG_LEVEL_ERROR_MI = "Set log level to 'Error'";
const string LOG_LEVEL_CRITICAL_MI = "Set log level to 'Critical'";
const string LOG_LEVEL_OFF_MI = "Set log level to 'Off'";

// logging callback menu texts
const string LOG_CALLBACK_MENU = "Logging Callback Menu";
const string LOG_CALLBACK_CORE_MI = "Activate log callback for Nanolib Core";
const string LOG_CALLBACK_CANOPEN_MI = "Activate log callback for CANopen module";
const string LOG_CALLBACK_ETHERCAT_MI = "Activate log callback for EtherCAT module";
const string LOG_CALLBACK_MODBUS_MI = "Activate log callback for Modbus module";
const string LOG_CALLBACK_REST_MI = "Activate log callback for REST module";
const string LOG_CALLBACK_USB_MI = "Activate log callback for USB/MSC module";
const string LOG_CALLBACK_DEACTIVATE_MI = "Deactivate current log callback";

// sampler menu texts
const string SAMPLER_EXAMPLE_MENU = "Sampler Example Menu";
const string SAMPLER_NORMAL_WO_NOTIFY_MI = "Sampler w/o Notification - Normal Mode";
const string SAMPLER_REPETETIVE_WO_NOTIFY_MI = "Sampler w/o Notification - Repetetive Mode";
const string SAMPLER_CONTINUOUS_WO_NOTIFY_MI = "Sampler w/o Notification - Continuous Mode";
const string SAMPLER_NORMAL_WITH_NOTIFY_MI = "Sampler with Notification - Normal Mode";
const string SAMPLER_REPETETIVE_WITH_NOTIFY_MI = "Sampler with Notification - Repetetive Mode";
const string SAMPLER_CONTINUOUS_WITH_NOTIFY_MI = "Sampler with Notification - Continuous Mode";

// motor example menu texts
const string MOTOR_EXAMPLE_MENU = "Motor Example Menu";
const string MOTOR_AUTO_SETUP_MI = "Initial commissioning - motor auto setup";
const string MOTOR_VELOCITY_MI = "Run a motor in profile velocity mode";
const string MOTOR_POSITIONING_MI = "Run a motor in positioning mode";

// profinet menu texts
const string PROFINET_EXAMPLE_MI = "ProfinetDCP example";

// main menu title
const string MAIN_MENU = "Nanolib Example Main";

/// @brief Structure for menu context
struct Context {
	size_t selectedOption; // the selected option of user
	string errorText; // the error text of last action (if error occured)
	LogLevel currentLogLevel; // holds the current log level
	NanoLibAccessor *nanolibAccessor; // holds the nanolib accessor
	vector<BusHardwareId> scannedBusHardwareIds; // vector holds found bus hardware ids
	vector<BusHardwareId> openableBusHardwareIds; // vector holds found bus hardware ids not yet opened
	vector<BusHardwareId> openBusHardwareIds; // vector holds opened bus hardware ids
	vector<DeviceId> scannedDeviceIds; // vector holds found devices of opened bus hardware
	vector<DeviceId> connectableDeviceIds; // vector holds found devices not yet connected
	vector<DeviceHandle> connectedDeviceHandles; // vector holds device handles of connected devices
	DeviceHandle activeDevice; // the current active device
	LogModule currentLogModule; // the log module currently used
	bool loggingCallbackActive; // flag for active logging callback
	bool waitForUserConfirmation; // flag to wait for user confirmation after a function has been executed
	LoggingCallbackExample *loggingCallback; // pointer to logging callback
	ScanBusCallbackExample *scanBusCallback; // pointer to scan bus callback
	DataTransferCallbackExample *dataTransferCallback; // pointer tor data transfer callback
	ColorModifier red; // color modifier for red forground color
	ColorModifier green; // color modifier for green forground color
	ColorModifier blue; // color modifier for blue forground color
	ColorModifier yellow; // color modifier for yellow forground color
	ColorModifier light_red; // color modifier for light red forground color
	ColorModifier light_green; // color modifier for light green forground color 
	ColorModifier light_blue; // color modifier for light blue forground color
	ColorModifier light_yellow; // color modifier for light yellow forground color
	ColorModifier dark_gray; // color modifier for dark gray foreground color
	ColorModifier def; // color modifier for the default color
	ColorModifier reset_all; // color modifier to reset everything to default
};

// highest byte = error number
inline string getErrorNumberString(const int64_t &number) {
	uint32_t bitMask = 0xff000000;
	uint8_t byteValue = (uint8_t)((number & bitMask) >> 24);
	string resultString;

	switch (byteValue) {
		case 0:
			resultString = string("    0: Watchdog Reset");
			break;
		case 1:
			resultString = string("    1: Input voltage (+Ub) too high");
			break;
		case 2:
			resultString = string("    2: Output current too high");
			break;
		case 3:
			resultString = string("    3: Input voltage (+Ub) too low");
			break;
		case 4:
			resultString = string("    4: Error at fieldbus");
			break;
		case 6:
			resultString = string("    6: CANopen only: NMT master takes too long to send Nodeguarding request");
			break;
		case 7:
			resultString = string("    7: Sensor 1 (see 3204h): Error through electrical fault or defective hardware");
			break;
		case 8:
			resultString = string("    8: Sensor 2 (see 3204h): Error through electrical fault or defective hardware");
			break;
		case 9:
			resultString = string("    9: Sensor 3 (see 3204h): Error through electrical fault or defective hardware");
			break;
		case 10:
			resultString = string("   10: Positive limit switch exceeded");
			break;
		case 11:
			resultString = string("   11: Negative limit switch exceeded");
			break;
		case 12:
			resultString = string("   12: Overtemperature error");
			break;
		case 13:
			resultString = string("   13: The values of object 6065h and 6066h were exceeded; a fault was triggered.");
			break;
		case 14:
			resultString = string("   14: Nonvolatile memory full. Controller must be restarted for cleanup work.");
			break;
		case 15:
			resultString = string("   15: Motor blocked");
			break;
		case 16:
			resultString = string("   16: Nonvolatile memory damaged; controller must be restarted for cleanup work.");
			break;
		case 17:
			resultString = string("   17: CANopen only: Slave took too long to send PDO messages.");
			break;
		case 18:
			resultString = string("   18: Sensor n (see 3204h), where n is greater than 3: Error through electrical fault or defective hardware");
			break;
		case 19:
			resultString = string("   19: CANopen only: PDO not processed due to a length error.");
			break;
		case 20:
			resultString = string("   20: CANopen only: PDO length exceeded.");
			break;
		case 21:
			resultString = string("   21: Restart the controller to avoid future errors when saving (nonvolatile memory full/corrupt).");
			break;
		case 22:
			resultString = string("   22: Rated current must be set (203Bh:01h/6075h).");
			break;
		case 23:
			resultString = string("   23: Encoder resolution, number of pole pairs and some other values are incorrect.");
			break;
		case 24:
			resultString = string("   24: Motor current is too high, adjust the PI parameters.");
			break;
		case 25:
			resultString = string("   25: Internal software error, generic.");
			break;
		case 26:
			resultString = string("   26: Current too high at digital output.");
			break;
		case 27:
			resultString = string("   27: CANopen only: Unexpected sync length.");
			break;
		case 30:
			resultString = string("   30: Error in speed monitoring: slippage error too large.");
			break;
		case 32:
			resultString = string("   32: Internal error: Correction factor for reference voltage missing in the OTP.");
			break;
		case 35:
			resultString = string("   35: STO Fault: STO was requested but not via both STO inputs");
			break;
		case 36:
			resultString = string("   36: STO Changeover: STO was requested but not via both STO inputs.");
			break;
		case 37:
			resultString = string("   37: STO Active: STO is active, it generates no torque or holding torque.");
			break;
		case 38:
			resultString = string("   38: STO Self-Test: Error during self-test of the firmware. Contact Nanotec.");
			break;
		case 39:
			resultString = string("   39: Error in the ballast configuration: Invalid/unrealistic parameters entered.");
			break;
		case 40:
			resultString = string("   40: Ballast resistor thermally overloaded.");
			break;
		case 41:
			resultString = string("   41: Only EtherCAT: Sync Manager Watchdog: The controller has not received any PDO data for an excessively long period of time.");
			break;
		case 46:
			resultString = string("   46: Interlock error: Bit 3 in 60FDh is set to 0, the motor may not start.");
			break;
		case 48:
			resultString = string("   48: Only CANopen: NMT status has been set to stopped.");
			break;
		default:
			resultString = string("   " + to_string(byteValue) + ": Unknown error number");
			break;
	}

	return resultString;
}

	// second highest byte = error class
inline string  getErrorClassString(const int64_t &number) {
	uint32_t bitMask = 0xff0000;
	uint8_t byteValue = (uint8_t)((number & bitMask) >> 16);
	string resultString;

	switch (byteValue) {
		case 1:
			resultString = string("    1: General error, always set in the event of an error.");
			break;
		case 2:
			resultString = string("    2: Current.");
			break;
		case 4:
			resultString = string("    4: Voltage.");
			break;
		case 8:
			resultString = string("    8: Temperature.");
			break;
		case 16:
			resultString = string("   16: Communication");
			break;
		case 32:
			resultString = string("   32: Relates to the device profile.");
			break;
		case 64:
			resultString = string("   64: Reserved, always 0.");
			break;
		case 128:
			resultString = string("  128: Manufacturer-specific.");
			break;
		default:
			resultString = string("  " + to_string(byteValue) + ": Unkonw error class.");
			break;
	}

	return resultString;
}

// lower 16 bit = error code
inline string getErrorCodeString(const int64_t &number) {
	uint32_t bitMask = 0xffff;
	uint16_t wordValue = (uint16_t)(number & bitMask);
	string resultString;

	switch (wordValue) {
		case 0x1000: 
			resultString = string("0x1000: General error.");
			break;
		case 0x2300: 
			resultString = string("0x2300: Current at the controller output too large.");
			break;
		case 0x3100: 
			resultString = string("0x3100: Overvoltage/undervoltage at controller input.");
			break;
		case 0x4200: 
			resultString = string("0x4200: Temperature error within the controller.");
			break;
		case 0x5440: 
			resultString = string("0x5440: Interlock error: Bit 3 in 60FDh is set to 0, the motor may not start .");
			break;
		case 0x6010: 
			resultString = string("0x6010: Software reset (watchdog).");
			break;
		case 0x6100: 
			resultString = string("0x6100: Internal software error, generic.");
			break;
		case 0x6320: 
			resultString = string("0x6320: Rated current must be set (203Bh:01h/6075h).");
			break;
		case 0x7110: 
			resultString = string("0x7110: Error in the ballast configuration: Invalid/unrealistic parameters entered.");
			break;
		case 0x7113: 
			resultString = string("0x7113: Warning: Ballast resistor thermally overloaded.");
			break;
		case 0x7121: 
			resultString = string("0x7121: Motor blocked.");
			break;
		case 0x7200: 
			resultString = string("0x7200: Internal error: Correction factor for reference voltage missing in the OTP.");
			break;
		case 0x7305: 
			resultString = string("0x7305: Sensor 1 (see 3204h) faulty.");
			break;
		case 0x7306: 
			resultString = string("0x7306: Sensor 2 (see 3204h) faulty.");
			break;
		case 0x7307: 
			resultString = string("0x7307: Sensor n (see 3204h), where n is greater than 2.");
			break;
		case 0x7600: 
			resultString = string("0x7600: Warning: Nonvolatile memory full or corrupt; restart the controller for cleanup work.");
			break;
		case 0x8100: 
			resultString = string("0x8100: Error during fieldbus monitoring.");
			break;
		case 0x8130: 
			resultString = string("0x8130: CANopen only: Life Guard error or Heartbeat error.");
			break;
		case 0x8200: 
			resultString = string("0x8200: CANopen only: Slave took too long to send PDO messages.");
			break;
		case 0x8210: 
			resultString = string("0x8210: CANopen only: PDO was not processed due to a length error.");
			break;
		case 0x8220: 
			resultString = string("0x8220: CANopen only: PDO length exceeded.");
			break;
		case 0x8240: 
			resultString = string("0x8240: CANopen only: unexpected sync length.");
			break;
		case 0x8400: 
			resultString = string("0x8400: Error in speed monitoring: slippage error too large.");
			break;
		case 0x8611: 
			resultString = string("0x8611: Position monitoring error: Following error too large.");
			break;
		case 0x8612: 
			resultString = string("0x8612: Position monitoring error: Limit switch exceeded.");
			break;
		default:
			resultString = string(to_string(wordValue) + ": Uknown error code.");
			break;
	}

	return resultString;
}

/// @brief Helper function to generate the proper bus hardware options
/// @param busHardwareId 
/// @return returns bus hardware options to use for busHardwareId
inline BusHardwareOptions createBusHardwareOptions(const BusHardwareId &busHardwareId) {
	// create new bus hardware options
	BusHardwareOptions busHwOptions;

	// now add all options necessary for opening the bus hardware
	if (busHardwareId.getProtocol() == BUS_HARDWARE_ID_PROTOCOL_CANOPEN) {
		// in case of CAN bus it is the baud rate
		busHwOptions.addOption(busHwOptionsDefaults.canBus.BAUD_RATE_OPTIONS_NAME,
							   busHwOptionsDefaults.canBus.baudRate.BAUD_RATE_1000K);

		if (busHardwareId.getBusHardware() == BUS_HARDWARE_ID_IXXAT) {
			// in case of HMS IXXAT we need also bus number
			busHwOptions.addOption(
				busHwOptionsDefaults.canBus.ixxat.ADAPTER_BUS_NUMBER_OPTIONS_NAME,
				busHwOptionsDefaults.canBus.ixxat.adapterBusNumber.BUS_NUMBER_0_DEFAULT);
		}

		if (busHardwareId.getBusHardware() == BUS_HARDWARE_ID_PEAK) {
			// in case of PEAK PCAN we need also bus number
			busHwOptions.addOption(
				busHwOptionsDefaults.canBus.peak.ADAPTER_BUS_NUMBER_OPTIONS_NAME,
				busHwOptionsDefaults.canBus.peak.adapterBusNumber.BUS_NUMBER_1_DEFAULT);
		}
	} else if (busHardwareId.getProtocol() == BUS_HARDWARE_ID_PROTOCOL_MODBUS_RTU) {
		// in case of Modbus RTU it is the serial baud rate
		busHwOptions.addOption(busHwOptionsDefaults.serial.BAUD_RATE_OPTIONS_NAME,
							   busHwOptionsDefaults.serial.baudRate.BAUD_RATE_19200);
		// and serial parity
		busHwOptions.addOption(busHwOptionsDefaults.serial.PARITY_OPTIONS_NAME,
							   busHwOptionsDefaults.serial.parity.EVEN);
	} else {
	}

	return busHwOptions;
}

/// @brief Removes leading and trailing white-space chars from string s
/// @param s - string to use (not changed)
/// @return returns the updated string
inline string trim(const string &s) {
	constexpr char whitespace[] = " \t\n\r";
	const size_t first = s.find_first_not_of(whitespace);

	return (first != string::npos) ? s.substr(first, (s.find_last_not_of(whitespace) - first + 1))
								   : string{};
}

/// @brief Checks if string starts with a number
/// @tparam T - typename for int
/// @param s - string to check
/// @return returns true or false
template <typename T = int> bool startsWithDigit(const string &s) {
	if (s.empty())
		return false;

	if (isdigit((unsigned char)s.front()))
		return true;

	return (((is_signed<T>::value && (s.front() == '-')) || (s.front() == '+'))
			&& ((s.size() > 1) && isdigit(s[1])));
}

/// @brief Converts a string to a number
/// @tparam T - typename for int
/// @param st - string to convert
/// @return returns either value of converted number or no value if text number cannot be converted
template <typename T = int> optional<T> stonum(const string &st) {
	const auto s = trim(st);
	bool ok = startsWithDigit<T>(s);

	auto v = T{};

	if (ok) {
		istringstream ss(s);

		ss >> v;
		ok = (ss.peek() == EOF);
	}

	return ok ? v : optional<T>{};
}

/// @brief Obtain a line of text from specified stream. Removes any existing data from input buffer
/// @param is - input stream
/// @param def - optional default text if no text entered
/// @return either valid input line or no value if problem obtaining input
inline optional<string> getline(istream &is, const string &def = "") {
	for (auto no = is.rdbuf()->in_avail(); no && is && isspace(is.peek()); is.ignore(), --no)
		;

	string ln;

	return getline(is, ln) ? (ln.empty() && !def.empty() ? def : ln)
						   : (is.clear(), optional<string>{});
}

/// @brief Obtain a line of text from console. 
/// @details Displays prompt text. If default text provided display within [..] after prompt
/// @param prm - optional prompt text to display first prm
/// @param def - optional default text
/// @return if no text entered returns entered text as type string. No error conditions. Only returns when valid data entered
inline auto getline(const string &prm = "", const string &def = "") {
	optional<string> o;

	do {
		cout << prm;
		if (!def.empty())
			cout << " [" << def << "]";

		cout << ": ";
		o = getline(cin, def);
	} while (!o.has_value() && (cout << "Invalid input" << endl));

	return *o;
}

/// @brief Extract next item of data from specified stream. Data must terminate with a white-space char
/// @tparam T Defaults to type string
/// @param is - stream from which to extract data
/// @return either valid extracted data or no value if problem extracting data
template <typename T = string> optional<T> getdata(istream &is) {
	auto i = T{};
	const bool b = (is >> i) && isspace(is.peek());

	for (is.clear(); is && !isspace(is.peek()); is.ignore())
		;
	return b ? i : optional<T>{};
}

/// @brief Obtains a number from specified stream in specified type
/// @tparam T Default of number type is int
/// @param is - stream from which to obtain number
/// @param wholeline - true if only one number per line (default), false if can have multiple numbers per line
/// @return either valid number of required type or no value if problem extracting data
template <typename T = int> auto getnum(istream &is, bool wholeline = true) {
	if (wholeline) {
		const auto o = getline(is);
		return o.has_value() ? stonum<T>(*o) : optional<T>{};
	}

	return getdata<T>(is);
}

/// @brief  Obtains a number from the console. 
/// @details First displays prompt text. If specified, number must be within 
///          the specified min..max range and range displayed as (...) after prm
/// @tparam T Default of number type is int
/// @param prm - optional prompt text to display first
/// @param nmin - optional minimum valid value
/// @param nmax - optional maximum valid value
/// @param wholeline - true if only one number per line (default), false if can have multiple numbers per line
/// @return returns when valid number entered
template <typename T = int>
auto getnum(const string &prm = "", T nmin = numeric_limits<T>::lowest(),
			T nmax = numeric_limits<T>::max(), bool wholeline = true) {
	const auto showdefs = [nmin, nmax]() {
		cout << " (";

		if (nmin != numeric_limits<T>::lowest() || is_unsigned<T>::value)
			cout << nmin;

		cout << " - ";

		if (nmax != numeric_limits<T>::max())
			cout << nmax;

		cout << ")";
	};

	optional<T> o;
	// clear screen, return value not needed
#ifdef _WIN32
    int result = system("CLS");
#else
    int result = system("clear");
#endif
    (void)result;
	
	cout << prm;

	if ((nmin != numeric_limits<T>::lowest()) || (nmax != numeric_limits<T>::max()))
		showdefs();

	cout << ": ";
	o = getnum<T>(cin, wholeline);

	// not a number or number out of bounds
	if ((!o.has_value()) || ((*o < nmin || *o > nmax))) {
		o = numeric_limits<size_t>::max();
	}

	return *o;
}

/// @brief Obtains a char from the specified stream
/// @param is - stream from which to obtain number
/// @param def - default char to return if no character obtained
/// @param wholeline - true if only one char per line (default), false if can have multiple chars per line
/// @return returns either valid character or no value if problem extracting data
inline optional<char> getchr(istream &is, char def = 0, bool wholeline = true) {
	if (wholeline) {
		if (auto o = getline(is); o.has_value()) {
			return (o->empty() && def ? def : ((o->size() == 1) ? o->front() : optional<char>{}));
		} else {
			return {};
		}
	}

	return getdata<char>(is);
}

/// @brief Obtains a char from the console. First displays prompt text
/// @param prm - optional prompt text to display first
/// @param valid - optional string containing valid values for the char. Displayed within (...)
/// @param def - optional default char to use if none entered. Displayed within [...]
/// @param wholeline - true if only one char per line (default), false if can have multiple chars per line
/// @return returns valid char. No error conditions. Only returns when valid char entered
inline auto getchr(const string &prm = "", const string &valid = "", char def = 0, bool wholeline = true) {
	const auto showopt = [&valid, def]() {
		cout << " (";

		for (size_t i = 0, s = valid.size(); i < s; ++i)
			cout << (i ? "/" : "") << valid[i];

		if (cout << ")"; def)
			cout << " [" << def << "]";
	};

	optional<char> o;

	do {
		if (cout << prm; !valid.empty())
			showopt();

		cout << " :";
		o = getchr(cin, def, wholeline);
	} while ((!o.has_value() || ((!valid.empty()) && (valid.find(*o) == string::npos)))
			 && (cout << "Invalid input" << endl));

	return *o;
}

/// @brief Displays error message.
/// @param ctx - menu context
/// @param errorString - general error or warning string, will be displayed in yellow
/// @param errorReasonString - Abort error from application or Nanolib, will be displayed in red
/// @return - returns the generated string
inline string handleErrorMessage(Context &ctx, const string &errorString, const string &errorReasonString = "") {
	ostringstream errorMessage;
	// color error string in light_yellow and reason in light_red
	errorMessage << ctx.light_yellow << errorString << ctx.light_red << errorReasonString <<  ctx.def;
	// for menu info
	ctx.errorText = errorMessage.str();
	// for current output, if waitForUserConfirmation is true
	// otherwise we can skip the output
	if (ctx.waitForUserConfirmation) {
		cout << errorMessage.str() << endl;
	}
	
	return errorMessage.str();
}

/// @brief Using directive for void function pointer
using f_type = void (*)(Context &ctx);

/// @brief Menu class for CLI Menu
class Menu {
private:
	/// @brief MenuItem contains a name and a pointer to a menu or a function
	struct MenuItem {
		string name;
		variant<f_type, Menu *> func;
		bool isActivce;
	};

	/// @brief Using directive for vector containing menu items
	using vmi = vector<MenuItem>;

public: 
	/// @brief Default constructor
	Menu();

	/// @brief Menu constructor with all params
	/// @param t - the menu title
	/// @param vm  - the menu items to show
	/// @param df - pointer to default function to use for dynamic menu
	Menu(const string &t, const vmi &vm, const f_type df = nullptr);

	/// @brief Gets the title of a menu
	/// @return - returns the title as string
	auto getTitle() const noexcept {
		return title;
	}

	/// @brief Set a title of a menu
	/// @param t - the title to use as string
	void setTitle(const string &t) {
		title = t;
	}

	/// @brief Get the configured default function for dynamic menu
	/// @return - returns a pointer to a function or nullptr
	auto getDefaultFunction() {
		return default_func;
	}

	void menu(Context &ctx) {
		menu(*this, ctx);
	}

	/// @brief Delete a menu item
	/// @param index - index of item in vector to delete
	/// @return - returns true if found and deleted
	bool eraseMenuItem(size_t index);

	/// @brief Delete all configured menu items 
	/// @return - returns true
	bool eraseAllMenuItems();

	/// @brief Add a menu item (no duplication check)
	/// @param menuItem - the menu item to add (append)
	/// @return - returns true
	bool appendMenuItem(const MenuItem &menuItem);

	/// @brief Inserts a menu item at index
	/// @param index - the position to insert the menu item
	/// @param menuItem - the menu item to insert
	/// @return - returns true if insert was successful
	bool insertMenuItem(size_t index, const MenuItem &menuItem);

	/// @brief Prints basic information for the user
	/// @param ctx - menu context
	/// @return - the complete string for output
	string printInfo(Context &ctx) const noexcept;

	/// @brief Get all found devices not yet connected
	/// @param ctx - menu context
	/// @return - returns a vector of DeviceId for all found devices not yet connected
	static vector<DeviceId> getConnectableDeviceIds(const Context &ctx);

	/// @brief Get all found bus harware ids not yet opened
	/// @param ctx - menu context
	/// @return - returns a vector of BusHardwareId for all found bus hardware ids not yet opened 
	static vector<BusHardwareId> getOpenableBusHwIds(const Context &ctx);

	/// @brief Sets the default menu items with given name and default function for dynamic menus
	/// @param menu - the menu to modify
	/// @param ctx - menu context
	static void setMenuItems(Menu &menu, Context &ctx);

	/// @brief Build the active device string for printInfo
	/// @param ctx - menu context
	/// @return - returns a string of the active device
	static string getActiveDeviceString(Context &ctx);

	/// @brief Build the number of found bus hardware for printInfo
	/// @param ctx - menu context
	/// @return - returns the number as string for the found bushardware
	static string getFoundBusHwString(Context &ctx);

	/// @brief Build the opened bus hardware string for printInfo
	/// @param ctx - menu context
	/// @return - returns a string of opened bus hardware
	static string getOpenedBusHwIdString(Context &ctx);

	/// @brief Build the number of found devices printInfo
	/// @param ctx - menu context
	/// @return - returns a string of the found devices
	static string getScannedDeviceIdsString(Context &ctx);

	/// @brief Build the connected device(s) string for printInfo
	/// @param ctx - menu context
	/// @return - returns a string of the connected device(s)
	static string getConnectedDevicesString(Context &ctx);

	/// @brief Build the callback logging string for printInfo
	/// @param ctx - menu context
	/// @return - returns a string of currently used log module for logging callback
	static string getCallbackLoggingString(Context &ctx);

	/// @brief Build the object dictionary string for printInfo
	/// @param ctx - menu context
	/// @return - returns a string current assigned object dictionary
	static string getObjectDictionaryString(Context &ctx);

	/// @brief Display the menu, wait and get user input
	/// @param currentMenu - the menu to display
	/// @param ctx - menu context
	/// @return - returns the user selected option
	static size_t showMenu(Menu &currentMenu, Context &ctx);

private:
	static void menu(Menu &menu, Context &ctx);

private:
	/// @brief Menu title
	string title;

	/// @brief Vector of menu items
	vmi menuItems;

	/// @brief function pointer for dynamic menu
	f_type default_func;
};

} // namespace nlc

