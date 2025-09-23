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
* @file   logging_callback_example.cpp
*
* @brief  Definition of logging callback class
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "logging_callback_example.hpp"

namespace nlc {

LoggingCallbackExample::~LoggingCallbackExample() {
		NanoLibAccessor *nanolibAccessor = getNanoLibAccessor();
		nanolibAccessor->unsetLoggingCallback();
}

// handle stuff here (e.g. write to file)
void LoggingCallbackExample::callback(const string &payload_str, const string &formatted_str, const string &logger_name,
				  const unsigned int log_level, const uint64_t time_since_epoch,
				  const size_t thread_id) {
    string fomrattedString = formatted_str;
    size_t pos;
#if defined(_WIN32)
    // formatted_str contains a line separator (\r\n on windows or \n on linux) at the end of
    // log message
    pos = fomrattedString.find("\r\n", 0);
#else
    pos = fomrattedString.find("\n", 0);
#endif // WIN32
    fomrattedString = fomrattedString.substr(0, pos);
    // do your callback stuff here ...
    // e.g. print to file or console
    cout << "----------------------------------------------------------------------------------"
            << endl;
    cout << "| Payload = '" << payload_str << "'" << endl;
    cout << "| Formatted string = '" << fomrattedString << "'" << endl;
    cout << "| Logger name = '" << logger_name << "'" << endl;
    cout << "| nlc_log_level = '" << LogLevelConverter::toString((LogLevel)log_level)
            << "'" << endl;
    cout << "| Local Time = '" << timeSinceEpochToLocaltimeString(time_since_epoch) << "'"
            << endl;
    cout << "| Thread id = '" << thread_id << "'" << endl;
    cout << "----------------------------------------------------------------------------------"
            << endl;
}


/**
 * @brief Used to convert time since epoch value to a local time string
 *
 * @param time_since_epoch_in_ms  - time in milliseconds
 *
 */
string LoggingCallbackExample::timeSinceEpochToLocaltimeString(uint64_t time_since_epoch_in_ms) {
    char buff[128];
    chrono::milliseconds dur(time_since_epoch_in_ms);
    chrono::time_point<chrono::system_clock> tp(dur);
    time_t t = chrono::system_clock::to_time_t(tp);
    static struct tm out_time;
    struct tm *out_timep = &out_time;
    size_t fractional_seconds = dur.count() % 1000;
#if defined(_WIN32)
    localtime_s(out_timep, &t);
#else
    localtime_r(&t, out_timep);
#endif // _WIN32
    strftime(buff, sizeof(buff), "%d-%m-%Y %H:%M:%S", out_timep);
    string resDate(string(buff) + ":" + to_string(fractional_seconds));
    return resDate;
}

} // namespace nlc