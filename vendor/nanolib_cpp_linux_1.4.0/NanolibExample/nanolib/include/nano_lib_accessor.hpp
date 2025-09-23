#pragma once

#include "bus_hardware_id.hpp"
#include "bus_hardware_options.hpp"
#include "bus_hw_options_defaults.hpp"
#include "device_id.hpp"
#include "nlc_callback.hpp"
#include "od_index.hpp"
#include "od_library.hpp"
#include "od_types.hpp"
#include "result.hpp"
#include "result_base.hpp"
#include "profinet_dcp.hpp"
#include "sampler_interface.hpp"
#include "nlc_log_level.hpp"
#include <memory>
#include <functional>
#include <chrono>

namespace nlc {
/**
 * @brief Interface class used for entry point to the nanolib.
 *
 *
 * Brief workflow:
 * -# Scan is recommended: NanoLibAccessor.listAvailableBusHardware()
 * -# Communication settings: BusHardwareOptions(...)
 * -# NanoLibAccessor.openBusHardwareWithProtocol(...)
 * -# NanoLibAccessor.scanDevices(...)
 * -# NanoLibAccessor.addDevice(...)
 * -# NanoLibAccessor.connectDevice(...)
 * -# NanoLibAccessor.disconnectDevice(...)
 * -# NanoLibAccessor.removeDevice(...)
 * -# NanoLibAccessor.closeBusHardware(...)
 *
 */
class NanoLibAccessor {
public:
	NanoLibAccessor() {
	}
	virtual ~NanoLibAccessor() {
	}

	/**
	 * @brief Sets the depth (level of detail) of logging.
	 * The default level is LogLevel::Info.
	 *
	 * @param LogLevel logging level
	 * 
	 * @see LogLevel
	 */
	virtual void setLoggingLevel(LogLevel level) = 0;

	/**
	 * @brief Sets a logging callback function pointer for a
	 * 		  specific module.
	 *
	 * @param NlcLoggingCallback  *callback - callback pointer
	 * @param const nlc::LogModule &logModule - logging module
	 * 
	 * @return void
	 * 
	 */
	virtual void setLoggingCallback(NlcLoggingCallback *callback, const nlc::LogModule &logModule) = 0;

	/**
	 * @brief Unsets a logging callback function pointer.
	 * 
	 * @return void
	 * 
	 */
	virtual void unsetLoggingCallback() = 0;

	/**
	 * @brief Lists available bus hardware
	 *
	 * @return ResultBusHwIds
	 */
	virtual ResultBusHwIds listAvailableBusHardware() = 0;

	/**
	 * @brief Establishes connection with a bus hardware
	 *
	 * @param busHwId Bus hardware id
	 * @param busHwOpt Bus configuration
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid openBusHardwareWithProtocol(const BusHardwareId &busHwId,
												   const BusHardwareOptions &busHwOpt)
		= 0;

	/**
	 * @brief Disconnects from bus hardware
	 *
	 * @param busHwId Bus hardware id
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid closeBusHardware(const BusHardwareId &busHwId) = 0;

	/**
	 * @brief Sets bus protocol specific state
	 *
	 * @param busHwId Bus hardware id
	 * @param state Bus specific string value
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid setBusState(const BusHardwareId &busHwId, const std::string &state) = 0;

	/**
	 * @brief Add a bus device described by deviceId to nanolib internal device list and return a
	 * DeviceHandle for it
	 *
	 * @param deviceId Device indentificator
	 *
	 * @return ResultDeviceHandle
	 */
	virtual ResultDeviceHandle addDevice(const DeviceId &deviceId) = 0;

	/**
	 * @brief Removes a bus device from nanolib internal list
	 *
	 * @param deviceHandle Handle of the bus device
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid removeDevice(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Device Id of specific device from nanolib internal list
	 *
	 * @param deviceHandle Handle of the bus device
	 *
	 * @return ResultDeviceIds
	 */
	virtual ResultDeviceId getDeviceId(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets all Device Ids from the nanolib internal list
	 *
	 * @return ResultDeviceIds
	 */
	virtual ResultDeviceIds getDeviceIds() = 0;

	/**
	 * @brief Establishes connection with DeviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultVoid Description (Result.hasError())
	 */
	virtual ResultVoid connectDevice(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Disconnects from DeviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid disconnectDevice(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Reboots device with DeviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid rebootDevice(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Vendor Id of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultInt
	 */
	virtual ResultInt getDeviceVendorId(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Product Code of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultInt
	 */
	virtual ResultInt getDeviceProductCode(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Name of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultString
	 */
	virtual ResultString getDeviceName(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Hardware version of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultString
	 */
	virtual ResultString getDeviceHardwareVersion(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Firmware build id of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultString
	 */
	virtual ResultString getDeviceFirmwareBuildId(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Bootloader build id of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultString
	 */
	virtual ResultString getDeviceBootloaderBuildId(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Serial number of the device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultString
	 */
	virtual ResultString getDeviceSerialNumber(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Unique ID (96 bit/12 bytes) of the device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultArrayByte
	 */
	virtual ResultArrayByte getDeviceUid(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Bootloader version of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultInt
	 */
	virtual ResultInt getDeviceBootloaderVersion(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets Hardware Group of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultInt
	 */
	virtual ResultInt getDeviceHardwareGroup(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets connection state of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultConnectionState
	 */
	virtual ResultConnectionState getConnectionState(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Checks connection state of bus device with deviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultConnectionState
	 */
	virtual ResultConnectionState checkConnectionState(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Gets device protocol specific state
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultString Bus specific string value
	 */
	virtual ResultString getDeviceState(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Sets device protocol specific state
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param state Bus specific string value
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid setDeviceState(const DeviceHandle deviceHandle, const std::string &state) = 0;

	/**
	 * @brief Scans for devices in a network
	 *
	 * @param BusHardwareId Available bus
	 * @param NlcScanBusCallback Callback object used to trace progress
	 *
	 * @return ResultDeviceIds Array of DeviceId
	 */
	virtual ResultDeviceIds scanDevices(const BusHardwareId &busHwId, NlcScanBusCallback *callback)
		= 0;

	/**
	 * @brief Gets the ProtocolSpecificAccessor object
	 *
	 * @param BusHardwareId Available bus
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid getProtocolSpecificAccessor(const BusHardwareId &busHwId) = 0;

	/**
	 * @brief Checks if connection with bus hardware is open
	 *
	 * @param BusHardwareId Available bus
	 *
	 * @return Boolean
	 */
	virtual bool isBusHardwareOpen(const BusHardwareId &busHardwareId) const = 0;

	/**
	 * @brief Reads a numeric value from controllers object dictionary
	 *
	 * @param DeviceHandle The device to read from
	 * @param OdIndex The index/sub-index to read from
	 *
	 * @return ResultInt The uninterpreted number value (can be signed, unsigned, fix16.16)
	 */
	virtual ResultInt readNumber(const DeviceHandle deviceHandle, const OdIndex odIndex) = 0;

	/**
	 * @brief Reads a string from controllers object dictionary
	 *
	 * @param DeviceHandle The device to read from
	 * @param OdIndex The index/sub-index to read from
	 *
	 * @return ResultString The uninterpreted string value (can be std::string or null)
	 */
	virtual ResultString readString(const DeviceHandle deviceHandle, const OdIndex odIndex) = 0;

	/**
	 * @brief Reads arbitrary bytes (domain object data) from controllers object dictionary
	 *
	 * @param DeviceHandle The device to read from
	 * @param odIndex The index/sub-index to read from
	 *
	 * @return ResultArrayByte The result array of bytes
	 */
	virtual ResultArrayByte readBytes(const DeviceHandle deviceHandle, const OdIndex odIndex) = 0;

	/**
	 * @brief Writes given number to the controllers object dictionary
	 *
	 * @param DeviceHandle The device to write to
	 * @param value The uninterpreted value (can be signed, unsigned, fix16.16)
	 * @param odIndex The index/sub-index to write to
	 * @param size Bits
	 *
	 * @result ResultVoid
	 */
	virtual ResultVoid writeNumber(const DeviceHandle deviceHandle, int64_t value,
								   const OdIndex odIndex, unsigned int bitLength)
		= 0;

	/**
	 * @brief Writes arbitrary bytes (domain object data) to the controllers object dictionary
	 *
	 * @param DeviceHandle The device to write to
	 * @param data Byte vector, array of bytes
	 * @param odIndex The index/sub-index to write to
	 *
	 * @result ResultVoid
	 */
	virtual ResultVoid writeBytes(const DeviceHandle deviceHandle, const std::vector<uint8_t> &data,
								  const OdIndex odIndex)
		= 0;

	/**
	 * @brief Reads numeric array from object dictionary
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param index Unsigned integer
	 *
	 * @return ResultArrayInt
	 */
	virtual ResultArrayInt readNumberArray(const DeviceHandle deviceHandle, const uint16_t index)
		= 0;

	/**
	 * @brief Updates firmware on controller
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param absoluteFilePath Path to file containing firmware data (uint8_t)
	 * @param NlcDataTransferCallback Callback object used to trace progress
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid uploadFirmwareFromFile(const DeviceHandle deviceHandle,
											  const std::string &absoluteFilePath,
											  NlcDataTransferCallback *callback)
		= 0;

	/**
	 * @brief Updates firmware on controller
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param fwData Array containing firmware data
	 * @param NlcDataTransferCallback Callback object used to trace progress
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid uploadFirmware(const DeviceHandle deviceHandle,
									  const std::vector<uint8_t> &fwData,
									  NlcDataTransferCallback *callback)
		= 0;

	/**
	 * @brief Updates bootloader on controller
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param bootloaderAbsoluteFilePath Path to file containing bootloader data (uint8_t)
	 * @param NlcDataTransferCallback Callback object used to trace progress
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid uploadBootloaderFromFile(const DeviceHandle deviceHandle,
												const std::string &bootloaderAbsoluteFilePath,
												NlcDataTransferCallback *callback)
		= 0;

	/**
	 * @brief Updates bootloader on controller
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param btData Bootloader data in array of uint8_t
	 * @param NlcDataTransferCallback Callback object used to trace progress
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid uploadBootloader(const DeviceHandle deviceHandle,
										const std::vector<uint8_t> &btData,
										NlcDataTransferCallback *callback)
		= 0;

	/**
	 * @brief Updates bootloader and firmware on the controller
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param bootloaderAbsoluteFilePath Path to file containing bootloader data (uint8_t)
	 * @param absoluteFilePath Path to file containing firmware data (uint8_t)
	 * @param NlcDataTransferCallback Callback object used to trace progress
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid uploadBootloaderFirmwareFromFile(
		const DeviceHandle deviceHandle, const std::string &bootloaderAbsoluteFilePath,
		const std::string &absoluteFilePath, NlcDataTransferCallback *callback)
		= 0;

	/**
	 * @brief Updates bootloader and firmware on the controller
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param btData Bootloader data in array of uint8_t
	 * @param fwData Firmware data in array of uint8_t
	 * @param NlcDataTransferCallback Callback object used to trace progress
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid
	uploadBootloaderFirmware(const DeviceHandle deviceHandle, const std::vector<uint8_t> &btData,
							 const std::vector<uint8_t> &fwData, NlcDataTransferCallback *callback)
		= 0;

	/**
	 * @brief Uploads NanoJ program to controller
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param absoluteFilePath Path to file containing NanoJ data (uint8_t)
	 * @param NlcDataTransferCallback Callback object used to trace progress
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid uploadNanoJFromFile(const DeviceHandle deviceHandle,
										   const std::string &absoluteFilePath,
										   NlcDataTransferCallback *callback)
		= 0;

	/**
	 * @brief Uploads NanoJ program to controller
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param vmmData NanoJ data in array of uint8_t
	 * @param NlcDataTransferCallback Callback object used to trace progress
	 *
	 * @return ResultVoid
	 */
	virtual ResultVoid uploadNanoJ(const DeviceHandle deviceHandle,
								   const std::vector<uint8_t> &vmmData,
								   NlcDataTransferCallback *callback)
		= 0;

	/**
	 * @brief Returns reference to the ObjectDictionary library
	 *
	 * @return OdLibrary&
	 */
	virtual OdLibrary &getObjectDictionaryLibrary() = 0;

	/**
	 * @brief Assigns ObjectDictionary to DeviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 * @param objectDictionary
	 *
	 * @return ResultObjectDictionary
	 */
	virtual ResultObjectDictionary assignObjectDictionary(const DeviceHandle deviceHandle,
														  const ObjectDictionary &objectDictionary)
		= 0;

	/**
	 * @brief Recursive directory scanning for a suitable Object Dictionary.
	 *        When a suitable Object Dictionary is found and successfully loaded, 
	 *        it assign to the device. 
	 *        Note: If a compatible object dictionary is already loaded in the object library, 
	 *        it will be used and the submitted directory will not be scanned.
	 * 
	 * @param deviceHandle - Handle of a device on the bus
	 * @param dictionariesLocationPath - Dictionaries location path
	 * 
	 * @return ResultObjectDictionary
	 */
	virtual ResultObjectDictionary
	autoAssignObjectDictionary(const DeviceHandle deviceHandle,
							   const std::string &dictionariesLocationPath)
		= 0;

	/**
	 * @brief Gets ObjectDictionary from DeviceHandle
	 *
	 * @param deviceHandle Handle of a device on the bus
	 *
	 * @return ResultObjectDictionary
	 */
	virtual ResultObjectDictionary getAssignedObjectDictionary(const DeviceHandle deviceHandle) = 0;

	/**
	 * @brief Returns reference to Profinet DCP interface
	 */
	virtual ProfinetDCP &getProfinetDCP() = 0;

	/**
	 * @brief Returns reference to Sampler interface
	 */
	virtual SamplerInterface &getSamplerInterface() = 0;

};
} // namespace nlc