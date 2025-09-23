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
* @file   logging_functions_example.cpp
*
* @brief  Definition of logging functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "logging_functions_example.hpp"

void setLogLevel(Context &ctx) {
	ctx.waitForUserConfirmation = false;
	size_t index = ctx.selectedOption;

	switch (index) { 
	case 1:
		ctx.nanolibAccessor->setLoggingLevel(LogLevel::Trace);
		ctx.currentLogLevel = LogLevel::Trace;
		break;
	case 2:
		ctx.nanolibAccessor->setLoggingLevel(LogLevel::Debug);
		ctx.currentLogLevel = LogLevel::Debug;
		break;
	case 3:
		ctx.nanolibAccessor->setLoggingLevel(LogLevel::Info);
		ctx.currentLogLevel = LogLevel::Info;
		break;
	case 4:
		ctx.nanolibAccessor->setLoggingLevel(LogLevel::Warning);
		ctx.currentLogLevel = LogLevel::Warning;
		break;
	case 5:
		ctx.nanolibAccessor->setLoggingLevel(LogLevel::Error);
		ctx.currentLogLevel = LogLevel::Error;
		break;
	case 6:
		ctx.nanolibAccessor->setLoggingLevel(LogLevel::Critical);
		ctx.currentLogLevel = LogLevel::Critical;
		break;
	case 7:
		ctx.nanolibAccessor->setLoggingLevel(LogLevel::Off);
		ctx.currentLogLevel = LogLevel::Off;
		break;
	default:
		ctx.nanolibAccessor->setLoggingLevel(LogLevel::Info);
		ctx.currentLogLevel = LogLevel::Info;
		break;
	}
}

void setLoggingCallback(Context &ctx) {
	ctx.waitForUserConfirmation = false;
	size_t index = ctx.selectedOption;

	switch (index) {
	case 1:
		ctx.nanolibAccessor->setLoggingCallback(ctx.loggingCallback, LogModule::NanolibCore);
		ctx.currentLogModule = LogModule::NanolibCore;
		ctx.loggingCallbackActive = true;
		break;
	case 2:
		ctx.nanolibAccessor->setLoggingCallback(ctx.loggingCallback, LogModule::NanolibCANopen);
		ctx.currentLogModule = LogModule::NanolibCANopen;
		ctx.loggingCallbackActive = true;
		break;
	case 3:
		ctx.nanolibAccessor->setLoggingCallback(ctx.loggingCallback, LogModule::NanolibEtherCAT);
		ctx.currentLogModule = LogModule::NanolibEtherCAT;
		ctx.loggingCallbackActive = true;
		break;
	case 4:
		ctx.nanolibAccessor->setLoggingCallback(ctx.loggingCallback, LogModule::NanolibModbus);
		ctx.currentLogModule = LogModule::NanolibModbus;
		ctx.loggingCallbackActive = true;
		break;
	case 5:
		ctx.nanolibAccessor->setLoggingCallback(ctx.loggingCallback, LogModule::NanolibRest);
		ctx.currentLogModule = LogModule::NanolibRest;
		ctx.loggingCallbackActive = true;
		break;
	case 6:
		ctx.nanolibAccessor->setLoggingCallback(ctx.loggingCallback, LogModule::NanolibUSB);
		ctx.currentLogModule = LogModule::NanolibUSB;
		ctx.loggingCallbackActive = true;
		break;
	default:
		ctx.nanolibAccessor->unsetLoggingCallback();
		ctx.loggingCallbackActive = false;
		break;
	}
}
