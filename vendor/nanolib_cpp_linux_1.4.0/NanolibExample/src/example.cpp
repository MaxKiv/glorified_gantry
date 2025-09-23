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
* @file   example.cpp
*
* @brief  Main function, definition of menu structure, signal handling etc.
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include <any>
#include <vector>
#include <thread>
#include <chrono>
#include <csignal>
#include "menu_utils.hpp"
#include "sampler_example.hpp"
#include "bus_functions_example.hpp"
#include "device_functions_example.hpp"
#include "logging_functions_example.hpp"
#include "od_interface_functions_example.hpp"
#include "profinet_functions_example.hpp"
#include "sampler_functions_example.hpp"
#include "motor_functions_example.hpp"

using namespace nlc;
using namespace std;
using namespace menu_color;

/// @brief Creates the connectDeviceMenu menu with dynamic entries linked to connectDevice function
/// @param ctx - the menu context
void buildConnectDeviceMenu(Context & ctx) {
	Menu connectDeviceMenu{DEVICE_CONNECT_MENU,
						{{ /* placeholder, dynamically filled */}}, connectDevice}; 

	connectDeviceMenu.menu(ctx);
}

/// @brief Creates the disconnectFromDeviceMenu menu with dynamic entries linked to disconnectDevice function
/// @param ctx - the menu context
void buildDisconnectDeviceMenu(Context & ctx) {
	Menu disconnectFromDeviceMenu{DEVICE_DISCONNECT_MENU, {{/* placeholder, dynamically filled */}}, disconnectDevice}; 

	disconnectFromDeviceMenu.menu(ctx);
}

/// @brief Creates the openBusHwMenu menu with dynamic entries linked to openBusHardware function
/// @param ctx - the menu context
void buildOpenBusHwMenu(Context & ctx) {
	Menu openBusHwMenu{BUS_HARDWARE_OPEN_MI,
					   {{/* placeholder, dynamically filled */}}, openBusHardware}; 

	openBusHwMenu.menu(ctx);
}

/// @brief Creates the closeBusHwMenu menu with dynamic entries linked to closeBusHardware function
/// @param ctx - the menu context
void buildCloseBusHwMenu(Context & ctx) {
	Menu closeBusHwMenu{BUS_HARDWARE_CLOSE_MI,
						{{/* placeholder, dynamically filled */}}, closeBusHardware};

	closeBusHwMenu.menu(ctx);
}

/// @brief Creates the buildSelectActiveDeviceMenu menu with dynamic entries linked to selectActiveDevice function
/// @param ctx - the menu context
void buildSelectActiveDeviceMenu(Context & ctx) {
	Menu selectActiveDeviceMenu{DEVICE_SELECT_ACTIVE_MENU,
						{{ /* placeholder, dynamically filled */}}, selectActiveDevice}; 

	selectActiveDeviceMenu.menu(ctx);
}


/// @brief Signal handler function
/// @param sig - the signal received
void signalHandler(int sig){
	cout << "Interrupt signal '";
	switch(sig) {
		case SIGABRT:
			cout << "SIGABRT";
			break;
		case SIGFPE:
			cout << "SIGFPE";
			break;
		case SIGILL:
			cout << "SIGILL";
			break;
		case SIGINT:
			cout << "SIGINT";
			break;
		case SIGSEGV:
			cout << "SIGSEGV";
			break;
		case SIGTERM:
			cout << "SIGTERM";
			break;
#ifdef SIGBREAK
		case SIGBREAK:
			cout << "SIGBREAK";
			break;
#endif
		default: 
			// should not get here
			break;
	}
	cout << "' received. Exiting ..." << endl;

	exit(sig);
}

/// @brief Main function
/// @return returns 0
int main() {
	// register signal handlers
	signal(SIGABRT, signalHandler);
	signal(SIGFPE, signalHandler);
	signal(SIGILL, signalHandler);
	signal(SIGINT, signalHandler);
	signal(SIGSEGV, signalHandler);
	signal(SIGTERM, signalHandler);
#ifdef SIGBREAK
	signal(SIGBREAK, signalHandler);
#endif

	Context context; // the menu context
	LoggingCallbackExample loggingCallback; // instantiate a logging callback 
	ScanBusCallbackExample scanBusCallback; // instantiate a scan bus callback 
	DataTransferCallbackExample dataTransferCallback; // instantiate a data transfer callback 

	// setup menu context
	context.currentLogLevel = LogLevel::Off; // no logging output at start
	context.nanolibAccessor = getNanoLibAccessor(); // get and store the nanolib accessor
	context.loggingCallbackActive = false; // no logging callback active
	context.loggingCallback = &loggingCallback; // store pointer to logging callback object
	context.scanBusCallback = &scanBusCallback; // store pointer to scan bus callback object
	context.dataTransferCallback = &dataTransferCallback; // store pointer to data transfer callback object
	context.waitForUserConfirmation = false; // flag to stop at end of a menu function, so the user can read the output
	// set some coloring options
	// this will work with most terminals in linux and cmd and pwsh in windows
	context.red.setCode(FG_RED);
	context.green.setCode(FG_GREEN);
	context.blue.setCode(FG_BLUE);
	context.yellow.setCode(FG_YELLOW);
	context.light_red.setCode(FG_LIGHT_RED);
	context.light_green.setCode(FG_LIGHT_GREEN);
	context.light_blue.setCode(FG_LIGHT_BLUE);
	context.light_yellow.setCode(FG_LIGHT_YELLOW);
	context.dark_gray.setCode(FG_DARK_GRAY);
	context.def.setCode(FG_DEFAULT);
	context.reset_all.setCode(RESET);
	
	// set log level to off
	context.nanolibAccessor->setLoggingLevel(LogLevel::Off);

	// build the motorMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu motorMenu{MOTOR_EXAMPLE_MENU, 
					{{MOTOR_AUTO_SETUP_MI, motorAutoSetup, false},
					 {MOTOR_VELOCITY_MI, executeProfileVelocityMode, false}, 
				   	 {MOTOR_POSITIONING_MI, executePositioningMode, false}}};

	// build the samplerMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu samplerMenu{SAMPLER_EXAMPLE_MENU, 
					{{SAMPLER_NORMAL_WO_NOTIFY_MI, executeSamplerWithoutNotificationNormalMode, false}, 
					 {SAMPLER_REPETETIVE_WO_NOTIFY_MI, executeSamplerWithoutNotificationRepetetiveMode, false}, 
					 {SAMPLER_CONTINUOUS_WO_NOTIFY_MI, executeSamplerWithoutNotificationContinuousMode, false},
					 {SAMPLER_NORMAL_WITH_NOTIFY_MI, executeSamplerWithNotificationNormalMode, false}, 
					 {SAMPLER_REPETETIVE_WITH_NOTIFY_MI, executeSamplerWithNotificationRepetetiveMode, false}, 
					 {SAMPLER_CONTINUOUS_WITH_NOTIFY_MI, executeSamplerWithNotificationContinuousMode, false}}};

	// build the logCallbackMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu logCallbackMenu{LOG_CALLBACK_MENU,
						 {{LOG_CALLBACK_CORE_MI, setLoggingCallback, false},
						  {LOG_CALLBACK_CANOPEN_MI, setLoggingCallback, false},
						  {LOG_CALLBACK_ETHERCAT_MI, setLoggingCallback, false},
						  {LOG_CALLBACK_MODBUS_MI, setLoggingCallback, false},
						  {LOG_CALLBACK_REST_MI, setLoggingCallback, false},
						  {LOG_CALLBACK_USB_MI, setLoggingCallback, false},
						  {LOG_CALLBACK_DEACTIVATE_MI, setLoggingCallback, false}}};

	// build the logLevelgMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu logLevelgMenu{LOG_LEVEL_MENU,
					 {{LOG_LEVEL_TRACE_MI, setLogLevel, false},
					  {LOG_LEVEL_DEBUG_MI, setLogLevel, false},
					  {LOG_LEVEL_INFO_MI, setLogLevel, false},
					  {LOG_LEVEL_WARN_MI, setLogLevel, false},
					  {LOG_LEVEL_ERROR_MI, setLogLevel, false},
					  {LOG_LEVEL_CRITICAL_MI, setLogLevel, false},
					  {LOG_LEVEL_OFF_MI, setLogLevel, false}}};

	// build the loggingMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu loggingMenu{LOGGING_MENU,
					 {{LOGGING_SET_LOG_LEVEL_MI, &logLevelgMenu, true}, 
					  {LOGGING_SET_LOG_CALLBACK_MI, &logCallbackMenu, true}}};

	// build the odAccessMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu odAccessMenu{OD_INTERFACE_MENU,
					  {{OD_ASSIGN_OD_MI, assignObjectDictionary, false},
					   {OD_READ_NUMBER_MI, readNumber, false},
					   {OD_READ_NUMBER_VIA_OD_MI, readNumberViaDictionaryInterface, false},
					   {OD_WRITE_NUMBER_MI, writeNumber, false},
					   {OD_WRITE_NUMBER_VIA_OD_MI, writeNumberViaDictionaryInterface, false},
					   {OD_READ_STRING_MI, readString, false},
					   {OD_READ_BYTES_MI, readArray, false}}};

	// build the deviceInfoMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu deviceInfoMenu{DEVICE_INFORMATION_MENU,
						{{DEVICE_GET_VENDOR_ID_MI, getDeviceVendorId, false},
						 {DEVICE_GET_PRODUCT_CODE_MI, getDeviceProductCode, false},
						 {DEVICE_GET_DEVICE_NAME_MI, getDeviceName, false},
						 {DEVICE_GET_HW_VERSION_MI, getDeviceHardwareVersion, false},
						 {DEVICE_GET_FW_BUILD_ID_MI, getDeviceFirmwareBuildId, false},
						 {DEVICE_GET_BL_BUILD_ID_MI, getDeviceBootloaderBuildId, false},
						 {DEVICE_GET_SERIAL_NUMBER_MI, getDeviceSerialNumber, false},
						 {DEVICE_GET_UNIQUE_ID_MI, getDeviceUid, false},
						 {DEVICE_GET_BL_VERSION_MI, getDeviceBootloaderVersion, false},
						 {DEVICE_GET_HW_GROUP_MI, getDeviceHardwareGroup, false},
						 {DEVICE_GET_CON_STATE_MI, getConnectionState, false}}};

	// build the deviceMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu deviceMenu{DEVICE_MENU,
					{{DEVICE_SCAN_MI, scanDevices, false},
					 {DEVICE_CONNECT_MENU, buildConnectDeviceMenu, false},
					 {DEVICE_DISCONNECT_MENU, buildDisconnectDeviceMenu, false},
					 {DEVICE_SELECT_ACTIVE_MENU, buildSelectActiveDeviceMenu, false},
					 {DEVICE_REBOOT_MI, rebootDevice, false},
					 {DEVICE_INFORMATION_MENU, &deviceInfoMenu, false},
					 {DEVICE_UPDATE_FW_MI, updateFirmware, false},
					 {DEVICE_UPDATE_BL_MI, updateBootloader, false},
					 {DEVICE_UPLOAD_NANOJ_MI, uploadNanoJ, false},
					 {DEVICE_RUN_NANOJ_MI, runNanoJ, false},
					 {DEVICE_STOP_NANOJ_MI, stopNanoJ, false},
					 {DEVICE_GET_ERROR_FIELD_MI, getErrorFields, false},
					 {DEVICE_RESTORE_ALL_DEFAULT_PARAMS_MI, restoreDefaults, false}}};

	// build the busHwMenu menu as a submenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu busHwMenu{BUS_HARDWARE_MENU,
				   {{BUS_HARDWARE_SCAN_MI, scanBusHardware, true},
					{BUS_HARDWARE_OPEN_MI, buildOpenBusHwMenu, false},
					{BUS_HARDWARE_CLOSE_MI, buildCloseBusHwMenu, false},
					{BUS_HARDWARE_CLOSE_ALL_MI, closeAllBusHardware, false}}};

	// build the mainMenu menu as a mainmenu
	// every menu has a title and menu items
	// every menu item has a name and a "pointer" to a function or another menu
	// on dynamic menus the function is used to build the underlaying menu
	Menu mainMenu{MAIN_MENU,	   
				  {{BUS_HARDWARE_MENU, &busHwMenu, true},
				   {DEVICE_MENU, &deviceMenu, false},
				   {OD_INTERFACE_MENU, &odAccessMenu, false},
				   {LOGGING_MENU, &loggingMenu, true},
				   {SAMPLER_EXAMPLE_MENU, &samplerMenu, false},
				   {MOTOR_EXAMPLE_MENU, &motorMenu, false},
				   {PROFINET_EXAMPLE_MI, &profinetDCPExample, false}}};

	// start the main menu
	mainMenu.menu(context);

	// close all opened bus hardware
	// connected devices will be disconnected / removed automatically
	closeAllBusHardware(context);

	// exit main program
	return 0;
}
