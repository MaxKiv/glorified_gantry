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
* @file   logging_callback_example.hpp
*
* @brief  Declaration of logging callback class
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#pragma once
#include <vector>
#include <string>
#include <iostream>
#include <chrono>
#include "nlc_callback.hpp"
#include "nlc_constants.hpp"
#include "accessor_factory.hpp"

using namespace nlc;
using namespace std;

namespace nlc {

/// @brief Implementation class of NlcLoggingCallback, handling the logging callback
class LoggingCallbackExample : public NlcLoggingCallback {
public:
	/// @brief Destructor for removing bound callback function pointer
	~LoggingCallbackExample() override;

	/// @brief Gets called whenever a log output is made by spdlog
	/// @param payload_str - the complete logging string
	/// @param formatted_str - the formatted logging string
	/// @param logger_name - name of the logger 
	/// @param log_level - log level
	/// @param time_since_epoch - timestamp in ms (since epoch)
	/// @param thread_id - thread id of logging call
	void callback(const string &payload_str, const string &formatted_str, const string &logger_name,
				  const unsigned int log_level, const uint64_t time_since_epoch,
				  const size_t thread_id) override;
private:
	/// @brief Helper class for converting time since epoch to local time
	/// @param time_since_epoch_in_ms 
	/// @return - returns the converted local time as string
	string timeSinceEpochToLocaltimeString(uint64_t time_since_epoch_in_ms);
};

} // namespace nlc