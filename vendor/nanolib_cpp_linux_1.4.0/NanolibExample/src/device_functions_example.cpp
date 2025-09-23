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
* @file   device_functions_example.cpp
*
* @brief  Definition of device specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "device_functions_example.hpp"

void scanDevices(Context &ctx) {
	ctx.waitForUserConfirmation = false;
	bool found = false;
	ctx.scannedDeviceIds.clear();

	// no bus hardware
	if (ctx.openBusHardwareIds.size() == 0) {
        handleErrorMessage(ctx, "No bus hardware available. Please scan and select a bus hardware first.");
		return;
	}

	// scan for every opened bushardware
	for (auto openBusHardwareId : ctx.openBusHardwareIds) {
		cout << "Scan devices for " << openBusHardwareId.getProtocol() << " (" << openBusHardwareId.getName() << ")" << endl;
		ResultDeviceIds resultDeviceIds
			= ctx.nanolibAccessor->scanDevices(openBusHardwareId, ctx.scanBusCallback);
		if (resultDeviceIds.hasError()) {
            handleErrorMessage(ctx, "Error during device scan: ", resultDeviceIds.getError());
			continue;
		}

		if (!resultDeviceIds.getResult().empty()) {
			found = true;
			for (auto newDeviceId : resultDeviceIds.getResult()) {
				ctx.scannedDeviceIds.emplace_back(newDeviceId);
			}
		}
	}

	if (!found) {
        handleErrorMessage(ctx, "No devices found. Please check your cabling, driver(s) and/or device(s).");
		return;
	}

	// update ctx.connectableDeviceIds
	ctx.connectableDeviceIds = Menu::getConnectableDeviceIds(ctx);
}

void connectDevice(Context &ctx) {
	ctx.waitForUserConfirmation = false;

	if (ctx.connectableDeviceIds.empty()) {
        handleErrorMessage(ctx, "No device available. Please scan for devices first.");
		return;
	}

	// check if selected device is already connected
	size_t index = ctx.selectedOption;
	DeviceId selectedDeviceId = ctx.connectableDeviceIds.at(index - 1);

	ResultDeviceHandle deviceHandleResult = ctx.nanolibAccessor->addDevice(selectedDeviceId);
	if (deviceHandleResult.hasError()) {
        handleErrorMessage(ctx, "Error during connectDevice (addDevice): ", deviceHandleResult.getError());
		return;
	}

	DeviceHandle deviceHandle = deviceHandleResult.getResult();

	ResultVoid resultVoid = ctx.nanolibAccessor->connectDevice(deviceHandle);
	if (resultVoid.hasError()) {
        handleErrorMessage(ctx, "Error during connectDevice: ", resultVoid.getError());
		ctx.nanolibAccessor->removeDevice(deviceHandle);
		return;
	}

	// store handle
	ctx.connectedDeviceHandles.push_back(deviceHandle);

	// update availableDeviceIds
	ctx.connectableDeviceIds = Menu::getConnectableDeviceIds(ctx);

	// update ctx.activeDevice to new connection
	ctx.activeDevice = deviceHandle;
}

void disconnectDevice(Context &ctx) {
	ctx.waitForUserConfirmation = false;
	size_t index = ctx.selectedOption;

	if (ctx.connectedDeviceHandles.empty()) {
        handleErrorMessage(ctx, "No device connected.");
		return;
	}

	// get selected device handle
	DeviceHandle closeDeviceHandle = ctx.connectedDeviceHandles.at(index - 1);

	// disconnect device in nanolib
	ResultVoid resultVoid = ctx.nanolibAccessor->disconnectDevice(closeDeviceHandle);
	if (resultVoid.hasError()) {
        handleErrorMessage(ctx, "Error during disconnectDevice: ", resultVoid.getError());
		return;
	}

	// remove device from nanolib
	resultVoid = ctx.nanolibAccessor->removeDevice(closeDeviceHandle);
	if (resultVoid.hasError()) {
        handleErrorMessage(ctx, "Error during disconnectDevice (removeDevice): ", resultVoid.getError());
		return;
	}

	// update ctx.connectedDeviceHandles
	auto it = find_if(ctx.connectedDeviceHandles.begin(), ctx.connectedDeviceHandles.end(), 
                       // comparison is done here   
                       [&](const DeviceHandle& e) { return e.equals(closeDeviceHandle); });
	if (it != ctx.connectedDeviceHandles.end()) {
		ctx.connectedDeviceHandles.erase(it);
	}

	// update ctx.connectableDeviceIds
	ctx.connectableDeviceIds = Menu::getConnectableDeviceIds(ctx);

	// clear ctx.activeDevice
	ctx.activeDevice = DeviceHandle();
}

void selectActiveDevice(Context &ctx) {
	ctx.waitForUserConfirmation = false;

	size_t index = ctx.selectedOption;

	if (ctx.connectedDeviceHandles.empty()) {
        handleErrorMessage(ctx, "No connected devices. Connect a device first.");
		return;
	}

	ctx.activeDevice = ctx.connectedDeviceHandles.at(index - 1);
}

void rebootDevice(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	}

	ResultVoid rebootResult = ctx.nanolibAccessor->rebootDevice(ctx.activeDevice);
	if (rebootResult.hasError()) {
        handleErrorMessage(ctx, "Error during rebootDevice: ", rebootResult.getError());
	}
}

void updateFirmware(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	}

	optional<string> inputPath;
	string deviceName = ctx.nanolibAccessor->getDeviceName(ctx.activeDevice).getResult();
	string firmwareBuildId = ctx.nanolibAccessor->getDeviceFirmwareBuildId(ctx.activeDevice).getResult();

	cout << "Current firmware Build Id: " << firmwareBuildId << endl;
	cout << "Please enter the full path to the firmware file" << endl << "(e.g. " << deviceName << "-FIR-vXXXX-BXXXXXXX.fw" << "): ";
	do {
		inputPath = getline(cin);
	} while (!inputPath.has_value());

	cout << "Do not interrupt the data connection or swich off the power until the update process has been finished!" << endl;
	ResultVoid uploadResult = ctx.nanolibAccessor->uploadFirmwareFromFile(ctx.activeDevice, inputPath.value(), ctx.dataTransferCallback);
	if (uploadResult.hasError()) {
        handleErrorMessage(ctx, "Error during updateFirmware: ", uploadResult.getError());
		return;
	}
	cout << endl;
}

void updateBootloader(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	}

	optional<string> inputPath;
	string deviceName = ctx.nanolibAccessor->getDeviceName(ctx.activeDevice).getResult();
	string bootloaderBuildId = ctx.nanolibAccessor->getDeviceBootloaderBuildId(ctx.activeDevice).getResult();
	string bootloaderVersion = to_string((ctx.nanolibAccessor->getDeviceBootloaderVersion(ctx.activeDevice).getResult()) >> 16);

	cout << "Current bootloader Build Id: " << bootloaderBuildId << endl;
	cout << "Bootloader version: " << bootloaderVersion << endl;
	cout << "Please enter the full path to the bootloader file: ";
	do {
		inputPath = getline(cin);
	} while (!inputPath.has_value());

	cout << "Do not interrupt the data connection or swich off the power until the update process has been finished!" << endl;
	ResultVoid uploadResult = ctx.nanolibAccessor->uploadBootloaderFromFile(ctx.activeDevice, inputPath.value(), ctx.dataTransferCallback);
	if (uploadResult.hasError()) {
        handleErrorMessage(ctx, "Error during updateBootloader: ", uploadResult.getError());
		return;
	}
	cout << endl;
}

void uploadNanoJ(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	}
	
	optional<string> inputPath;
	cout << "Please enter the full path to the NanoJ file (e.g. vmmcode.usr" << "): ";
	do {
		inputPath = getline(cin);
	} while (!inputPath.has_value());

	cout << "Do not interrupt the data connection or swich off the power until the update process has been finished!" << endl;
	ResultVoid uploadResult = ctx.nanolibAccessor->uploadNanoJFromFile(ctx.activeDevice, inputPath.value(), ctx.dataTransferCallback);
	if (uploadResult.hasError()) {
        handleErrorMessage(ctx, "Error during uploadNanoJ: ", uploadResult.getError());
		return;
	}
	cout << endl;
	cout << "Use runNanoJ menu option to re-start the uploaded NanoJ program." << endl;
}

void runNanoJ(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	}

    // check for errors
	ResultInt errorResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odNanoJError);
	if (errorResult.hasError()) {
        handleErrorMessage(ctx, "Error during runNanoJ: ", errorResult.getError());
		return;
	}

	if (errorResult.getResult() != 0) {
        handleErrorMessage(ctx, "Failed to start NanoJ program - NanoJ error code is set: ", to_string(errorResult.getResult()));
		return;
	}

	// write start to NanoJ control object (0x2300)
	ResultVoid writeNumberResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x1, odNanoJControl, 32);
	if (writeNumberResult.hasError()) {
        handleErrorMessage(ctx, "Error during runNanoJ: ", writeNumberResult.getError());
		return;
	}
		
	// start might take some time (up to 200ms)
	this_thread::sleep_for(chrono::milliseconds(250));

	// check if running and no error
	errorResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odNanoJError);
	if (errorResult.hasError()) {
        handleErrorMessage(ctx, "Error during runNanoJ: ", errorResult.getError());
		return;
	}

	if (errorResult.getResult() != 0) {
        handleErrorMessage(ctx, "Error during runNanoJ - program exited with error: ", to_string(errorResult.getResult()));
		return;
	}

	// check if program is still running, stopped or has error
	ResultInt readNumberResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odNanoJStatus);
	if (readNumberResult.hasError()) {
        handleErrorMessage(ctx, "Error during runNanoJ: ", readNumberResult.getError());
		return;
	}

	if (readNumberResult.getResult() == 0) {
		cout << "NanoJ program stopped ..." << endl;
	} else if (readNumberResult.getResult() == 1) {
		cout << "NanoJ program running ..." << endl;
	} else {
		cout << "NanoJ program exited with error." << endl;
	}
}

void stopNanoJ(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	}

	ResultVoid writeNumberResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x00, odNanoJControl, 32);
	if (writeNumberResult.hasError()) {
        handleErrorMessage(ctx, "Error during stopNanoJ: ", writeNumberResult.getError());
		return;
	}

	// stop might take some time
	this_thread::sleep_for(chrono::milliseconds(50));

	ResultInt readNumberResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odNanoJStatus);
	if (readNumberResult.hasError()) {
        handleErrorMessage(ctx, "Error during stopNanoJ: ", readNumberResult.getError());
		return;
	}

	if (readNumberResult.getResult() == 0) {
		cout << "NanoJ program stopped ..." << endl;
	} else if (readNumberResult.getResult() == 1) {
		cout << "NanoJ program still running ..." << endl;
	} else {
		cout << "NanoJ program exited with error: " << ctx.nanolibAccessor->readNumber(ctx.activeDevice, odNanoJError).getResult();
	}
}

void getDeviceVendorId(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultInt resultInt = ctx.nanolibAccessor->getDeviceVendorId(ctx.activeDevice);

	if (resultInt.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceVendorId: ", resultInt.getError());
		return;
	}

	cout << "Device vendor id = '" << to_string(resultInt.getResult()) << "'" << endl;
}

void getDeviceProductCode(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultInt resultInt = ctx.nanolibAccessor->getDeviceProductCode(ctx.activeDevice);

	if (resultInt.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceProductCode: ", resultInt.getError());
		return;
	}

	cout << "Device product code = '" << to_string(resultInt.getResult()) << "'" << endl;
}

void getDeviceName(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultString resultString = ctx.nanolibAccessor->getDeviceName(ctx.activeDevice);

	if (resultString.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceName: ", resultString.getError());
		return;
	}

	cout << "Device name = '" << resultString.getResult() << "'" << endl;
}

void getDeviceHardwareVersion(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultString resultString = ctx.nanolibAccessor->getDeviceHardwareVersion(ctx.activeDevice);

	if (resultString.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceHardwareVersion: ",resultString.getError());
		return;
	}

	cout << "Device hardware version = '" << resultString.getResult() << "'" << endl;
}

void getDeviceFirmwareBuildId(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultString resultString = ctx.nanolibAccessor->getDeviceFirmwareBuildId(ctx.activeDevice);

	if (resultString.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceFirmwareBuildId: ", resultString.getError());
		return;
	}

	cout << "Device firmware build id = '" << resultString.getResult() << "'" << endl;
}

void getDeviceBootloaderBuildId(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultString resultString
		= ctx.nanolibAccessor->getDeviceBootloaderBuildId(ctx.activeDevice);

	if (resultString.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceBootloaderBuildId: ", resultString.getError());
		return;
	}

	cout << "Device bootloader build id = '" << resultString.getResult() << "'" << endl;
}

void getDeviceSerialNumber(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultString resultString = ctx.nanolibAccessor->getDeviceSerialNumber(ctx.activeDevice);

	if (resultString.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceSerialNumber: ", resultString.getError());
		return;
	}

	cout << "Device serial number = '" << resultString.getResult() << "'" << endl;
}

void getDeviceUid(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultArrayByte resultArray = ctx.nanolibAccessor->getDeviceUid(ctx.activeDevice);

	if (resultArray.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceUid: ", resultArray.getError());
		return;
	}

	// convert byte array (vector) to hex string
	string s;
	s.reserve(resultArray.getResult().size()); // two digits per character
	static constexpr char hex[] = "0123456789ABCDEF";
	for (uint8_t c : resultArray.getResult())
    {
        s.push_back(hex[c / 16]);
        s.push_back(hex[c % 16]);
    }

	cout << "Device unique id = '" << s << "'" << endl;
}

void getDeviceBootloaderVersion(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultInt resultInt = ctx.nanolibAccessor->getDeviceBootloaderVersion(ctx.activeDevice);

	if (resultInt.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceBootloaderVersion: ", resultInt.getError());
		return;
	}

	cout << "Device bootloader version = '" << to_string(resultInt.getResult() >> 16) << "'" << endl;
}

void getDeviceHardwareGroup(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultInt resultInt = ctx.nanolibAccessor->getDeviceHardwareGroup(ctx.activeDevice);

	if (resultInt.hasError()) {
        handleErrorMessage(ctx, "Error during getDeviceHardwareGroup: ", resultInt.getError());
		return;
	}

	cout << "Device hardware group = '" << to_string(resultInt.getResult()) << "'" << endl;
}

void getConnectionState(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	ResultConnectionState resultConState
		= ctx.nanolibAccessor->getConnectionState(ctx.activeDevice);

	if (resultConState.hasError()) {
        handleErrorMessage(ctx, "Error during getConnectionState: ", resultConState.getError());
		return;
	}

	string connectionState = "unkown";

	switch (resultConState.getResult()) {
	case DeviceConnectionStateInfo::Connected:
		connectionState = "Connected";
		break;
	case DeviceConnectionStateInfo::Disconnected:
		connectionState = "Disconnected";
		break;
	case DeviceConnectionStateInfo::ConnectedBootloader: 
		connectionState = "Connected to bootloader";
		break;
	default:
		connectionState = "unknown";
		break;
	}

	cout << "Device connection state = '" << connectionState << "'" << endl;
}

void getErrorFields(Context &ctx) {
	ctx.waitForUserConfirmation = true;
	
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "", "No active device set. Select an active device first.");
		return;
	} 

	ResultInt errorNumberResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odErrorCount);
	if (errorNumberResult.hasError()) {
		handleErrorMessage(ctx, "Error during getErrorField: ", errorNumberResult.getError());
		return;
	}

	if (errorNumberResult.getResult() == 0) {
		cout << endl << "Currently there are no errors." << endl;
		return;
	}

	auto numberOfErrors = (uint8_t)(errorNumberResult.getResult());
	cout << "Currently there are " << std::to_string(numberOfErrors) << " errors." << endl << endl;

	// go through every error field (max 8)
	for (uint8_t i = 1; i <= numberOfErrors; i++) {
		OdIndex currentErrorField = OdIndex(odErrorStackIndex, i);

		errorNumberResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, currentErrorField);
		if (errorNumberResult.hasError()) {
			handleErrorMessage(ctx, "Error during getErrorField: ", errorNumberResult.getError());
			return;
		}

		// decode error field
		cout << "- Error Number [" << to_string(i) << "] = " << getErrorNumberString(errorNumberResult.getResult()) << endl;
		cout << "- Error Class  [" << to_string(i) << "] = " << getErrorClassString(errorNumberResult.getResult()) << endl;
		cout << "- Error Code   [" << to_string(i) << "] = " << getErrorCodeString(errorNumberResult.getResult()) << endl << endl;
	} 
}

void restoreDefaults(Context &ctx) {
	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "", "No active device set. Select an active device first.");
		return;
	} 

	// read Additional position encoder resolution interface #1
	auto posEncoderResResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odPosEncoderIncrementsInterface1);
	if (!posEncoderResResult.hasError()) {
		cout << "Position encoder resolution - encoder increments feedback interface #1 = " << posEncoderResResult.getResult() << endl;
	}

	// read Additional position encoder resolution interface #2
	posEncoderResResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odPosEncoderIncrementsInterface2);
	if (!posEncoderResResult.hasError()) {
		cout << "Position encoder resolution - encoder increments feedback interface #2 = " << posEncoderResResult.getResult() << endl;
	}
	
	// read Additional position encoder resolution interface #3
	posEncoderResResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odPosEncoderIncrementsInterface3);
	if (!posEncoderResResult.hasError()) {
		cout << "Position encoder resolution - encoder increments feedback interface #3 = " << posEncoderResResult.getResult() << endl;
	}

	// set interface values to zero
	ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0, odPosEncoderIncrementsInterface1, 32);
	ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0, odPosEncoderIncrementsInterface2, 32);
	ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0, odPosEncoderIncrementsInterface3, 32);
	
	auto subModeSelectResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odMotorDriveSubmodeSelect);
	cout << "Motor drive submode select = " << subModeSelectResult.getResult() << endl;

	// set motor drive submode select to zero
	ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0, odMotorDriveSubmodeSelect, 32);

	// Save all parameters to non-volatile memory
	ResultVoid writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 1702257011, odStoreAllParams, 32);
	if (writeResult.hasError()) {
		handleErrorMessage(ctx, "Error during restoreDefaults: ", writeResult.getError());
		return;
	}

	// wait until write has completed
	do {
		auto storeResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice,  odStoreAllParams);
		if (storeResult.getResult() == 1) {
			break;
		}
	} while(true);

	// reboot current active device
	cout << "Rebooting ..." << endl;
	ResultVoid rebootResult = ctx.nanolibAccessor->rebootDevice(ctx.activeDevice);
	if (rebootResult.hasError()) {
		handleErrorMessage(ctx, "Error during restoreDefaults: ", rebootResult.getError());
	}

	// Restore all default parameters
	cout << "Restoring all default parameters ..." << endl;
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 1684107116, odRestoreAllDefParams, 32);
	if (writeResult.hasError()) {
		handleErrorMessage(ctx, "Error during restoreDefaults: ", writeResult.getError());
		return;
	}

	// Restore tuning default parameters
	cout << "Restoring tuning default parameters ..." << endl;
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 1684107116, odRestoreTuningDefParams, 32);
	if (writeResult.hasError()) {
		handleErrorMessage(ctx, "Error during restoreDefaults: ", writeResult.getError());
		return;
	}

	// reboot current active device
	cout << "Rebooting ..." << endl;
	rebootResult = ctx.nanolibAccessor->rebootDevice(ctx.activeDevice);
	if (rebootResult.hasError()) {
		handleErrorMessage(ctx, "Error during restoreDefaults: ", rebootResult.getError());
	}

	cout << "All done. Check for errors." << endl;
}
