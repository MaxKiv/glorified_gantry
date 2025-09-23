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
* @file   bus_functions_example.cpp
*
* @brief  Definition of bus hardware specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "bus_functions_example.hpp"

void scanBusHardware(Context &ctx) {
	ctx.waitForUserConfirmation = false;

	ResultBusHwIds result = ctx.nanolibAccessor->listAvailableBusHardware();
	if (result.hasError()) {
        handleErrorMessage(ctx, "Error during bus scan: ", result.getError());
		return;
	}

	ctx.scannedBusHardwareIds = result.getResult();

	if (ctx.scannedBusHardwareIds.empty()) {
        handleErrorMessage(ctx, "No bus hardware found. Please check your cabling, driver and/or devices.");
		return;
	}
		
	ctx.openableBusHardwareIds = Menu::getOpenableBusHwIds(ctx);
}

void openBusHardware(Context &ctx) {
	ctx.waitForUserConfirmation = false;

	if (ctx.openableBusHardwareIds.empty()) {
        handleErrorMessage(ctx, "No bus hardware availalbe. Please do a scan first.");
		return;
	}

	// get selected option
	size_t index = ctx.selectedOption;
	BusHardwareId busHwId = ctx.openableBusHardwareIds.at(index - 1);

	// check if already openen
	for (auto openBusHwId : ctx.openBusHardwareIds) {
		if (openBusHwId.equals(busHwId)) {
            handleErrorMessage(ctx, "Bus hardware " + busHwId.getName() + " alread open.");
			return;
		}
	}

	BusHardwareOptions busHwOptions = createBusHardwareOptions(busHwId);
	ResultVoid resultVoid = ctx.nanolibAccessor->openBusHardwareWithProtocol(busHwId, busHwOptions);
	if (resultVoid.hasError()) {
        handleErrorMessage(ctx, "Error during openBusHardware: ", resultVoid.getError());
		return;
	}

	ctx.openBusHardwareIds.emplace_back(busHwId);

	// update available bus hardware ids for opening
	ctx.openableBusHardwareIds = Menu::getOpenableBusHwIds(ctx);
}

void closeBusHardware(Context &ctx) {
	ctx.waitForUserConfirmation = false;
	size_t index = ctx.selectedOption;
	ostringstream oss;

	if (ctx.openBusHardwareIds.empty()) {
        handleErrorMessage(ctx, "No open bus hardware found.");
		return;
	}

	// get selected option from menu
	BusHardwareId closeBusHardwareId = ctx.openBusHardwareIds.at(index - 1);

	// update ctx.connectedDeviceHandles
	// remove connected device handles from connectedDeviceHandles
	// it will be removed from nanolib connection in next step
	auto deviceHandleIt = find_if(ctx.connectedDeviceHandles.begin(), ctx.connectedDeviceHandles.end(),
	[&](const DeviceHandle &e) { return closeBusHardwareId.equals(ctx.nanolibAccessor->getDeviceId(e).getResult().getBusHardwareId()); });
	if (deviceHandleIt != ctx.connectedDeviceHandles.end()) {
		// reset active device handle if it belongs to bus hardware to close
		if (ctx.activeDevice.get() == deviceHandleIt->get()) {
			ctx.activeDevice = DeviceHandle();
		}

		// remove from connectedDeviceHandles
		ctx.connectedDeviceHandles.erase(deviceHandleIt);
	}

	// remove available device ids with matching BusHardwareId from ctx.connectableDeviceIds
	auto deviceIdIt = find_if(ctx.connectableDeviceIds.begin(), ctx.connectableDeviceIds.end(),
	[&](const DeviceId &e) { return closeBusHardwareId.equals(e.getBusHardwareId()); });
	if (deviceIdIt != ctx.connectableDeviceIds.end()) {
		ctx.connectableDeviceIds.erase(deviceIdIt);
	}

	// remove available device ids with matching BusHardwareId from ctx.scannedDeviceIds
	deviceIdIt = find_if(ctx.scannedDeviceIds.begin(), ctx.scannedDeviceIds.end(),
	[&](const DeviceId &e) { return closeBusHardwareId.equals(e.getBusHardwareId()); });
	if (deviceIdIt != ctx.scannedDeviceIds.end()) {
		ctx.scannedDeviceIds.erase(deviceIdIt);
	}

	// close the bus hardware in nanolib
	// connected devices will automatically be disconnected and removed
	ResultVoid resultVoid = ctx.nanolibAccessor->closeBusHardware(closeBusHardwareId);
	if (resultVoid.hasError()) {
        handleErrorMessage(ctx,  "Error during closeBusHardware: ", resultVoid.getError());
		return;
	}
	
	// update ctx.openBusHardwareIds
	auto busHardwareIdIt = find_if(ctx.openBusHardwareIds.begin(), ctx.openBusHardwareIds.end(),
					  [&](const BusHardwareId &e) { return e.equals(closeBusHardwareId); });
	if (busHardwareIdIt != ctx.openBusHardwareIds.end()) {
		ctx.openBusHardwareIds.erase(busHardwareIdIt);
	}

	// no bus hardware opened
	if (ctx.openBusHardwareIds.empty()) {
		// no bus hardware open -> clean scanned devices list
		ctx.scannedDeviceIds = vector<DeviceId>();
		// clear ctx.activeDevice 
		ctx.activeDevice = DeviceHandle();
	}

	// update ctx.openableBusHardwareIds
	ctx.openableBusHardwareIds = Menu::getOpenableBusHwIds(ctx);
}

void closeAllBusHardware(Context &ctx) {
	ctx.waitForUserConfirmation = false;
	ostringstream oss;

	// check for open bus hardware
	if (ctx.openBusHardwareIds.empty()) {
        handleErrorMessage(ctx, "No open bus hardware found.");
		return;
	}

	// go through all opened bus hardware
	for (auto openBusHardwareId : ctx.openBusHardwareIds) {
		// update ctx.connectedDeviceHandles
		// remove connected device handles from openDeviceHandles
		// it will be removed from nanolib connection in next step
		auto deviceHandleIt = find_if(ctx.connectedDeviceHandles.begin(), ctx.connectedDeviceHandles.end(),
		[&](const DeviceHandle &e) { return openBusHardwareId.equals(ctx.nanolibAccessor->getDeviceId(e).getResult().getBusHardwareId()); });
		if (deviceHandleIt != ctx.connectedDeviceHandles.end()) {
			ctx.connectedDeviceHandles.erase(deviceHandleIt);
		}

		// remove available device ids with matching BusHardwareId from ctx.connectableDeviceIds
		auto deviceIdIt = find_if(ctx.connectableDeviceIds.begin(), ctx.connectableDeviceIds.end(),
		[&](const DeviceId &e) { return openBusHardwareId.equals(e.getBusHardwareId()); });
		if (deviceIdIt != ctx.connectableDeviceIds.end()) {
			ctx.connectableDeviceIds.erase(deviceIdIt);
		}

		// close bus hardware in nanolib
		ResultVoid resultVoid = ctx.nanolibAccessor->closeBusHardware(openBusHardwareId);
		if (resultVoid.hasError()) {
			oss << handleErrorMessage(ctx, "Error during closeBusHardware: ", resultVoid.getError()) << endl;
			continue;
		}
	}

	ctx.errorText = oss.str();
	ctx.openBusHardwareIds = vector<BusHardwareId>();

	// no bus hardware open -> clean scanned devices list
	ctx.scannedDeviceIds = vector<DeviceId>();

	// update ctx.openableBusHardwareIds
	ctx.openableBusHardwareIds = Menu::getOpenableBusHwIds(ctx);

	// clear ctx.activeDevice 
	ctx.activeDevice = DeviceHandle();
}
