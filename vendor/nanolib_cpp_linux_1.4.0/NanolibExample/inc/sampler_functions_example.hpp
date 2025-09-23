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
* @file   sampler_functions_example.hpp
*
* @brief  Declarations for sampler specific functions
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#pragma once
#include "menu_utils.hpp"
#include "sampler_example.hpp"

using namespace nlc;
using namespace std;

/// @brief Function to execute the sampler example in normal mode without notification callback
/// @param ctx - menu context
void executeSamplerWithoutNotificationNormalMode(Context &ctx);

/// @brief Function to execute the sampler example in repetitive mode without notification callback
/// @param ctx - menu context
void executeSamplerWithoutNotificationRepetetiveMode(Context &ctx);

/// @brief Function to execute the sampler example in continuous mode without notification callback
/// @param ctx - menu context
void executeSamplerWithoutNotificationContinuousMode(Context &ctx);

/// @brief Function to execute the sampler example in normal mode with notification callback
/// @param ctx - menu context
void executeSamplerWithNotificationNormalMode(Context &ctx);

/// @brief Function to execute the sampler example in repetitive mode with notification callback
/// @param ctx - menu context
void executeSamplerWithNotificationRepetetiveMode(Context &ctx);

/// @brief Function to execute the sampler example in continuous mode with notification callback
/// @param ctx - menu context
void executeSamplerWithNotificationContinuousMode(Context &ctx);
