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
* @file   device_functions_example.hpp
*
* @brief  Declarations for device specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#pragma once
#include <thread>
#include <chrono>
#include "menu_utils.hpp"

using namespace nlc;
using namespace std;

/// @brief Scans for valid devices on all opened bus hardware
/// @param ctx - menu context
void scanDevices(Context &ctx);

/// @brief Adds device and connects to the selected device (ctx.selectedOption) within Nanolib
/// @param ctx - menu context
void connectDevice(Context &ctx);

/// @brief Disconnect device and removes to the selected device (ctx.selectedOption) within Nanolib
/// @param ctx - menu context
void disconnectDevice(Context &ctx);

/// @brief Select the device to use for all device specific functions in Nanolib
/// @param ctx - menu context
void selectActiveDevice(Context &ctx);

/// @brief Reboots the current active device
/// @param ctx - menu context
void rebootDevice(Context &ctx);

/// @brief Update the firmware of the current active device
/// @param ctx - menu context
void updateFirmware(Context &ctx);

/// @brief Update the bootloader of the current active device
/// @param ctx - menu context
void updateBootloader(Context &ctx);

/// @brief Upload a compiled NanoJ binary to the current active device
/// @param ctx - menu context
void uploadNanoJ(Context &ctx);

/// @brief Executes the NanoJ programm on the current active device if available
/// @param ctx - menu context
void runNanoJ(Context &ctx);

/// @brief Stops the NanoJ programm on the current active device if available
/// @param ctx - menu context
void stopNanoJ(Context &ctx);

/// @brief Read and output the device vendor id of the current active device
/// @param ctx - menu context
void getDeviceVendorId(Context &ctx);

/// @brief Read and output the product code of the current active device
/// @param ctx - menu context
void getDeviceProductCode(Context &ctx);

/// @brief Read and output the device name of the current active device
/// @param ctx - menu context
void getDeviceName(Context &ctx);

/// @brief Read and output the hardware version of the current active device
/// @param ctx - menu context
void getDeviceHardwareVersion(Context &ctx);

/// @brief Read and output the firmware build id of the current active device
/// @param ctx - menu context
void getDeviceFirmwareBuildId(Context &ctx);

/// @brief Read and output the bootloader build id of the current active device
/// @param ctx - menu context
void getDeviceBootloaderBuildId(Context &ctx);

/// @brief Read and output the serial number of the current active device
/// @param ctx - menu context
void getDeviceSerialNumber(Context &ctx);

/// @brief Read and output the device unique id of the current active device
/// @param ctx - menu context
void getDeviceUid(Context &ctx);

/// @brief Read and output the bootloeader version of the current active device
/// @param ctx - menu context
void getDeviceBootloaderVersion(Context &ctx);

/// @brief Read and output the hardware group of the current active device
/// @param ctx - menu context
void getDeviceHardwareGroup(Context &ctx);

/// @brief Read and output the connection state of the current active device
/// @param ctx - menu context
void getConnectionState(Context &ctx);

/// @brief Read and output error-stack
/// @param ctx - menu context
void getErrorFields(Context &ctx);

/// @brief Reset encoder resolution interfaces, reset drive mode selection and finally restore all default parameters
/// @param ctx - menu context
void restoreDefaults(Context &ctx);


