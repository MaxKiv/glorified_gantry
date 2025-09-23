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
* @file   od_interface_functions_example.hpp
*
* @brief  Declarations for object dictionary interface specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#pragma once
#include "menu_utils.hpp"

using namespace nlc;
using namespace std;

/// @brief Function to read a number (no interpretion of the data possible)
/// @param ctx - menu context
void readNumber(Context &ctx);

/// @brief Function to a string (string might be zero)
/// @param ctx - menu context
void readString(Context &ctx);

/// @brief Function to read an array (no interpretion of the data possible)
/// @param ctx - menu context
void readArray(Context &ctx);

/// @brief Function to write a number with certain length
/// @param ctx - menu context
void writeNumber(Context &ctx);

/// @brief Assign a valid object dictionary to the current active device
/// @param ctx - menu context
void assignObjectDictionary(Context &ctx);

/// @brief Function to read a number (with interpretion of the data)
/// @param ctx - menu context
void readNumberViaDictionaryInterface(Context &ctx);

/// @brief Function to write a number (no length has to be defined)
/// @param ctx - menu context
void writeNumberViaDictionaryInterface(Context &ctx);
