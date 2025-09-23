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
* @file   motor_functions_example.cpp
*
* @brief  Definition of motor specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "motor_functions_example.hpp"

void motorAutoSetup(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "", "No active device set. Select an active device first.");
		return;
	} 

	cout << endl << ctx.light_yellow;
	cout << "Please note the following requirements for performing the auto-setup: " << endl;
	cout << "- The motor must be unloaded." << endl;
	cout << "- The motor must not be touched." << endl;
	cout << "- The motor must be able to rotate freely in any direction." << endl;
	cout << "- No NanoJ program may be running." << ctx.def << endl;

	cout << "Do you want to continue? ";
	string result = getline("[y/n]", "y");
	if (result != "y") {
		return;
	}

	// stop a possibly running NanoJ program
	ResultVoid writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x00, odNanoJControl, 32);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// switch the state machine to "voltage enabled"
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// set mode of operation to auto-setup
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0xFE, odModeOfOperation, 8);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during motorAutoSetup: ", writeResult.getError());
		return;
	}

	// switch on
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x07, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// switch the state machine to "enable operation"
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x0F, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// run auto setup
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x1F, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	cout << "Moto auto setup is running, please wait ..." << endl;

	// wait until auto setup is finished, check status word
	do {
		ResultInt readNumberResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odStatusWord);
		if (readNumberResult.hasError()) {
			handleErrorMessage(ctx, "Error during motorAutoSetup: ", writeResult.getError());
			return;
		}

		// finish if bit 12, 9, 5, 4, 2, 1, 0 set
		if ((readNumberResult.getResult() & 0x1237) == 0x1237) {
			break;
		}
	} while (true);

	// reboot current active device
	cout << "Rebooting ..." << endl;
	ResultVoid rebootResult = ctx.nanolibAccessor->rebootDevice(ctx.activeDevice);
	if (rebootResult.hasError()) {
        handleErrorMessage(ctx, "Error during motorAutoSetup: ", rebootResult.getError());
		return;
	}
	cout << "Motor auto setup finished." << endl;
}

void executeProfileVelocityMode(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "", "No active device set. Select an active device first.");
		return;
	} 

	cout << "This example lets the motor run in Profile Velocity mode ..." << endl;

	// stop a possibly running NanoJ program
	ResultVoid writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x00, odNanoJControl, 32);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// choose Profile Velocity mode
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x03, odModeOfOperation, 8);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// set the desired speed in rpm (60)
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x3C, odTargetVelocity, 32);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// switch the state machine to "operation enabled"
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x07, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x0F, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}
	cout << "Motor is running clockwise ..." << endl;
	
	// let the motor run for 3s
	this_thread::sleep_for(chrono::milliseconds(3000));

	// stop the motor
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// set the desired speed in rpm (60), now counterclockwise
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, -0x3C, odTargetVelocity, 32);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}

	// start the motor
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x0F, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}
	cout << "Motor is running counterclockwise ..." << endl;

	// let the motor run for 3s
	this_thread::sleep_for(chrono::milliseconds(3000));

	// stop the motor
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executeProfileVelocityMode: ", writeResult.getError());
		return;
	}
}

void executePositioningMode(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "", "No active device set. Select an active device first.");
		return;
	} 

	cout << "This example lets the motor run in Profile Position mode ..." << endl;

	// stop a possibly running NanoJ program
	ResultVoid writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x00, odNanoJControl, 32);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx,  "Error during executePositioningMode: ", writeResult.getError());
		return;
	}

	// choose Profile Position mode
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x01, odModeOfOperation, 8);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}

	// set the desired speed in rpm (60)
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x3C, odProfileVelocity, 32);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}

	// set the desired target position (36000)
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x8CA0, odTargetPosition, 32);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx,"Error during executePositioningMode: ", writeResult.getError());
		return;
	}

	// switch the state machine to "operation enabled"
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x07, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x0F, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}

	// move the motor to the desired target position relatively
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x5F, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}
	cout << "Motor is running clockwise until position is reached ..." << endl;
	while(true) {
		auto readResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odStatusWord);
		if (readResult.hasError()) {
            handleErrorMessage(ctx, "Error during executePositioningMode: ", readResult.getError());
			// try to stop motor
			ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
			break;
		}

		if ((readResult.getResult() & 0x1400) == 0x1400) {
			break;
		}
	}

	// stop the motor
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}

	// set the desired target position (-36000)
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, -0x8CA0, odTargetPosition, 32);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ",  writeResult.getError());
		return;
	}

	// state machine operation enabled
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x0F, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}

	// move the motor to the desired target position relatively
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x5F, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}
	cout << "Motor is running counterclockwise until position is reached ..." << endl;
	while(true) {
		auto readResult = ctx.nanolibAccessor->readNumber(ctx.activeDevice, odStatusWord);
		if (readResult.hasError()) {
            handleErrorMessage(ctx, "Error during executePositioningMode: ", readResult.getError());
			// try to stop motor
			ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
			break;
		}

		if ((readResult.getResult() & 0x1400) == 0x1400) {
			break;
		}
	}

	// stop the motor
	writeResult = ctx.nanolibAccessor->writeNumber(ctx.activeDevice, 0x06, odControlWord, 16);
	if (writeResult.hasError()) {
        handleErrorMessage(ctx, "Error during executePositioningMode: ", writeResult.getError());
		return;
	}
}
