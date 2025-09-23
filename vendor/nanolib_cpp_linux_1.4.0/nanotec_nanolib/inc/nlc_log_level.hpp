#pragma once

#include <string>

namespace nlc {

#define LOG_LEVEL_TRACE		0
#define LOG_LEVEL_DEBUG 	1
#define LOG_LEVEL_INFO		2
#define LOG_LEVEL_WARN		3
#define LOG_LEVEL_ERROR		4
#define LOG_LEVEL_CRITICAL 	5
#define LOG_LEVEL_OFF 		6

/**
 * @brief Depth (level of detail) of logging
 * At the default logging depth (Info), logs will include informational, 
 * warning, error and critical messages.
 */
enum class LogLevel : unsigned int {
	Trace    = LOG_LEVEL_TRACE, /*!< The greatest possible depth of detail (expect huge logfiles). Entering and leaving functionality is included here. */
	Debug    = LOG_LEVEL_DEBUG, /*!< Logs include debug information - intermediate results, content sent or received, etc. */
	Info     = LOG_LEVEL_INFO,  /*!< Default level, informational messages */
	Warning  = LOG_LEVEL_WARN,  /*!< Warning messages */
	Error    = LOG_LEVEL_ERROR, /*!< Errors is a level that includes messages describing a serious problem that prevents the execution of the current algorithm from continuing. */
	Critical = LOG_LEVEL_CRITICAL, /*!< Critical errors will result in programm termination */  
	Off      = LOG_LEVEL_OFF,   /*!< Logging is turned off, nothing is logged at all */
	n_levels
};

/**
 * @brief Logging module enum class, containing all known log modules
 */
enum class LogModule : unsigned int {
	NanolibCore = 0,
	NanolibCANopen,
	NanolibModbus,
	NanolibEtherCAT,
	NanolibRest,
	NanolibUSB
};

/**
 * @brief LogLevelConverter class, returning a LogLevel as std::string
 */
class LogLevelConverter {
public:
	static std::string toString(nlc::LogLevel logLevel) {
		switch (logLevel) {
			case nlc::LogLevel::Trace:
				return "Trace";
			case nlc::LogLevel::Debug:
				return "Debug";
			case nlc::LogLevel::Info:
				return "Info";
			case nlc::LogLevel::Warning:
				return "Warning";
			case nlc::LogLevel::Error:
				return "Error";
			case nlc::LogLevel::Critical:
				return "Critical";
			case nlc::LogLevel::Off:
				return "Off";
			default:
				return "Unkown log level";
		}
	}
};

/**
 * @brief LogModuleConverter class, returning a log module as std::string
 */
class LogModuleConverter {
public:
	static std::string toString(nlc::LogModule logModule) {
		switch (logModule) {
			case nlc::LogModule::NanolibCore:
				return "nanolib_core";
			case nlc::LogModule::NanolibCANopen:
				return "nanolib_canopen";
			case nlc::LogModule::NanolibModbus:
				return "nanolib_modbus";
			case nlc::LogModule::NanolibEtherCAT:
				return "nanolib_ethercat";
			case nlc::LogModule::NanolibRest:
				return "nanolib_restful_api";
			case nlc::LogModule::NanolibUSB:
				return "nanolib_usbmsc";
			default:
				return "nanolib_unknown";
		}
	}
};

}
