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
* @file   sampler_functions_example.cpp
*
* @brief  Definition of sampler specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "sampler_functions_example.hpp"

void executeSamplerWithoutNotificationNormalMode(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "In normal mode the number of samples can be configured." << endl;
	cout << "In this example the sampler will run for 5 samples" << endl;

	SamplerExample samplerExample(ctx, ctx.activeDevice);
	samplerExample.processSamplerWithoutNotificationNormal();

	cout << "Finished" << endl;
}

void executeSamplerWithoutNotificationRepetetiveMode(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "In repetetive mode the sampler runs until stopped." << endl;
	cout << "In this example the sampler will run for 4 iterations and then stop." << endl;

	SamplerExample samplerExample(ctx, ctx.activeDevice);
	samplerExample.processSamplerWithoutNotificationRepetitive();

	cout << "Finished" << endl;
}

void executeSamplerWithoutNotificationContinuousMode(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "In continuous mode the sampler runs until stopped." << endl;
	cout << "In this example the sampler will run for 10 samples and then stop." << endl;

	SamplerExample samplerExample(ctx, ctx.activeDevice);
	samplerExample.processSamplerWithoutNotificationContinuous();

	cout << "Finished" << endl;
}

void executeSamplerWithNotificationNormalMode(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "In normal mode the number of samples can be configured." << endl;
	cout << "In this example the sampler will run for 5 samples" << endl;

	SamplerExample samplerExample(ctx, ctx.activeDevice);
	samplerExample.processSamplerWithNotificationNormal();

	cout << "Finished" << endl;
}

void executeSamplerWithNotificationRepetetiveMode(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "In repetetive mode the sampler runs until stopped." << endl;
	cout << "In this example the sampler will run for 4 iterations and then stop." << endl;

	SamplerExample samplerExample(ctx, ctx.activeDevice);
	samplerExample.processSamplerWithNotificationRepetitive();

	cout << "Finished" << endl;
}

void executeSamplerWithNotificationContinuousMode(Context &ctx) {
	ctx.waitForUserConfirmation = true;

	if (ctx.activeDevice.get() == 0) {
        handleErrorMessage(ctx, "No active device set. Select an active device first.");
		return;
	} 

	cout << "In continuous mode the sampler runs until stopped." << endl;
	cout << "In this example the sampler will run for 10 samples and then stop." << endl;

	SamplerExample samplerExample(ctx, ctx.activeDevice);
	samplerExample.processSamplerWithNotificationContinuous();

	cout << "Finished" << endl;
}
