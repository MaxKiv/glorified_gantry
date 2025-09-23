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
* @file   profinet_functions_example.cpp
*
* @brief  Definition of Profinet specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "profinet_functions_example.hpp"

void profinetDCPExample(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.openBusHardwareIds.empty()) {
        handleErrorMessage(ctx, "No hardware bus available. Open a proper hardware bus first.");
		return;
	}

	bool foundProfinetDevice = false;

	// Check service availability - Npcap/WinPcap driver required
	ProfinetDCP &profinetDCP = ctx.nanolibAccessor->getProfinetDCP();

	// search for profinet on every open bus hardware
	for (auto openBusHwId : ctx.openBusHardwareIds) {
		ResultVoid serviceResult = profinetDCP.isServiceAvailable(openBusHwId);
		if (serviceResult.hasError()) {
			// ignore
			continue;
		}

		// service available - scan for profinet devices
		cout << "Scanning " << openBusHwId.getName() << " for Profinet devices..." << endl;
		ResultProfinetDevices resultProfinetDevices	= profinetDCP.scanProfinetDevices(openBusHwId);

		if (resultProfinetDevices.hasError()
			&& (resultProfinetDevices.getErrorCode() != NlcErrorCode::TimeoutError)) {
			cout << "Error during profinetDCPExample: " << resultProfinetDevices.getError() << endl;
			continue;
		}

		const vector<ProfinetDevice> &profinetDevices = resultProfinetDevices.getResult();
		const size_t numberOfProfinetDevices = profinetDevices.size();

		if (numberOfProfinetDevices < 1) {
			continue;
		}

		foundProfinetDevice = true;
		cout << numberOfProfinetDevices << " Profinet device(s) found:" << endl;
		for (const auto &profinetDevice : profinetDevices) {

			cout << "IP: " << ((profinetDevice.ipAddress >> 24) & 0x000000FF) << "."
					<< ((profinetDevice.ipAddress >> 16) & 0x000000FF) << "."
					<< ((profinetDevice.ipAddress >> 8) & 0x000000FF) << "."
					<< (profinetDevice.ipAddress & 0x000000FF)
					<< "\tName: " << profinetDevice.deviceName << std::endl;

			// Checking the IP address against the context of the current network configuration
			const auto resultValid
				= profinetDCP.validateProfinetDeviceIp(openBusHwId, profinetDevice);
			std::cout << "\tDevice IP is " << (resultValid.hasError() ? " not " : "")
					<< "valid in the current network." << endl;

			// Trying to blink the device
			const auto resultBlink = profinetDCP.blinkProfinetDevice(openBusHwId, profinetDevice);
			cout << "\tBlink the device ";
			if (resultBlink.hasError())
				cout << "failed with error: " << resultBlink.getError();
			else
				cout << "succeeded.";
			cout << endl;
		}
	}

	if (!foundProfinetDevice) {
        handleErrorMessage(ctx, "No Profinet devices found.");
	}
}
