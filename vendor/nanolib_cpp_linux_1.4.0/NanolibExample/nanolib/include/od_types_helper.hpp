#pragma once

#include "od_types.hpp"

#include <algorithm>

namespace nlc {
/**
* @brief Custom data types
*/
class OdTypesHelper {
public:
	OdTypesHelper() = delete;
	~OdTypesHelper() = delete;
	OdTypesHelper(const OdTypesHelper &) = delete;
	OdTypesHelper(const OdTypesHelper &&) = delete;
	OdTypesHelper &operator=(const OdTypesHelper &) = delete;
	OdTypesHelper &operator=(const OdTypesHelper &&) = delete;
public:
	static ObjectCode uintToObjectCode(unsigned int objectCode) {
		switch (objectCode) {
		case static_cast<unsigned int>(ObjectCode::Deftype):
			return ObjectCode::Deftype;
		case static_cast<unsigned int>(ObjectCode::Defstruct):
			return ObjectCode::Defstruct;
		case static_cast<unsigned int>(ObjectCode::Var):
			return ObjectCode::Var;
		case static_cast<unsigned int>(ObjectCode::Array):
			return ObjectCode::Array;
		case static_cast<unsigned int>(ObjectCode::Record):
			return ObjectCode::Record;
		default:
			return ObjectCode::Null;
		}
	}

	static bool isNumericDataType(ObjectEntryDataType dataType) {
		return objectEntryDataTypeBitLength(dataType) != 0;
	}

	static bool isDefstructIndex(uint16_t typeNum) {
		return ((typeNum >= 0x0020) && (typeNum <= 0x005F))
			   || ((typeNum >= 0x0060) && (typeNum <= 0x025F) && ((typeNum & 0x0020) == 0x0000));
	}

	static bool isDeftypeIndex(uint16_t typeNum) {
		return ((typeNum >= 0x0001) && (typeNum <= 0x001F))
			   || ((typeNum >= 0x0060) && (typeNum <= 0x025F) && ((typeNum & 0x0020) == 0x0020));
	}

	static bool isComplexDataType(ObjectEntryDataType dataType) {
		return (dataType == ObjectEntryDataType::Complex)
			   || isDefstructIndex(static_cast<uint16_t>(dataType));
	}

	static ObjectEntryDataType uintToObjectEntryDataType(uint16_t objectDataType) {

		if (objectDataType <= static_cast<unsigned int>(ObjectEntryDataType::TimeDifference)) {
			return static_cast<ObjectEntryDataType>(objectDataType);
		} else if (objectDataType == static_cast<unsigned int>(ObjectEntryDataType::Domain)) {
			return ObjectEntryDataType::Domain;
		} else if (objectDataType <= 0x001F) {
				return ObjectEntryDataType::Simple;
		} else if (objectDataType > 0x025F) {
			return ObjectEntryDataType::Invalid;
		} else {
			switch (objectDataType) {
			case static_cast<uint16_t>(ObjectEntryDataType::PdoCommunicationParameter):
				return ObjectEntryDataType::PdoCommunicationParameter;
			case static_cast<uint16_t>(ObjectEntryDataType::PdoMapping):
				return ObjectEntryDataType::PdoMapping;
			case static_cast<uint16_t>(ObjectEntryDataType::Identity):
				return ObjectEntryDataType::Identity;
			case static_cast<uint16_t>(ObjectEntryDataType::SyncmgrSynchronization):
				return ObjectEntryDataType::SyncmgrSynchronization;
			case static_cast<uint16_t>(ObjectEntryDataType::UserStorageArea):
				return ObjectEntryDataType::UserStorageArea;
			case static_cast<uint16_t>(ObjectEntryDataType::SamplerConfig):
				return ObjectEntryDataType::SamplerConfig;
			case static_cast<uint16_t>(ObjectEntryDataType::SamplerTimebase):
				return ObjectEntryDataType::SamplerTimebase;
			case static_cast<uint16_t>(ObjectEntryDataType::PdiInput):
				return ObjectEntryDataType::PdiInput;
			case static_cast<uint16_t>(ObjectEntryDataType::PdiOutput):
				return ObjectEntryDataType::PdiOutput;
			case static_cast<uint16_t>(ObjectEntryDataType::ClosedLoopControllerParameter):
				return ObjectEntryDataType::ClosedLoopControllerParameter;
			case static_cast<uint16_t>(ObjectEntryDataType::OpenLoopControllerParameter):
				return ObjectEntryDataType::OpenLoopControllerParameter;
			case static_cast<uint16_t>(ObjectEntryDataType::SsiEncoder):
				return ObjectEntryDataType::SsiEncoder;
			case static_cast<uint16_t>(ObjectEntryDataType::N2):
				return ObjectEntryDataType::N2;
			case static_cast<uint16_t>(ObjectEntryDataType::N4):
				return ObjectEntryDataType::N4;
			case static_cast<uint16_t>(ObjectEntryDataType::V2):
				return ObjectEntryDataType::V2;
			case static_cast<uint16_t>(ObjectEntryDataType::X2):
				return ObjectEntryDataType::X2;
			case static_cast<uint16_t>(ObjectEntryDataType::X4):
				return ObjectEntryDataType::X4;
			case static_cast<uint16_t>(ObjectEntryDataType::InterpolationTimePeriod):
				return ObjectEntryDataType::InterpolationTimePeriod;
			case static_cast<uint16_t>(ObjectEntryDataType::InterpolationDataConfiguration):
				return ObjectEntryDataType::InterpolationDataConfiguration;
			case static_cast<uint16_t>(ObjectEntryDataType::VelocityAccelerationDeceleration):
				return ObjectEntryDataType::VelocityAccelerationDeceleration;
			default:
				return ObjectEntryDataType::Unknown;
			}
		}
	}

	/**
	 * @brief Convert ODDatatype to String
	 *
	 * @param odDataType
	 * @return std::string
	 */
	static std::string objectEntryDataTypeToString(ObjectEntryDataType odDataType) {
		switch (odDataType) {
		case ObjectEntryDataType::Boolean:
			return "BOOLEAN";
		case ObjectEntryDataType::Integer8:
			return "INTEGER8";
		case ObjectEntryDataType::Integer16:
			return "INTEGER16";
		case ObjectEntryDataType::Integer32:
			return "INTEGER32";
		case ObjectEntryDataType::Unsigned8:
			return "UNSIGNED8";
		case ObjectEntryDataType::Unsigned16:
			return "UNSIGNED16";
		case ObjectEntryDataType::Unsigned32:
			return "UNSIGNED32";
		case ObjectEntryDataType::Real32:
			return "REAL32";
		case ObjectEntryDataType::VisibleString:
			return "VISIBLE_STRING";
		case ObjectEntryDataType::OctetString:
			return "OCTET_STRING";
		case ObjectEntryDataType::UnicodeString:
			return "UNICODE_STRING";
		case ObjectEntryDataType::TimeOfDay:
			return "TIME_OF_DAY";
		case ObjectEntryDataType::TimeDifference:
			return "TIME_DIFFERENCE";
		case ObjectEntryDataType::Domain:
			return "DOMAIN";
		// Complex types
		case ObjectEntryDataType::PdoCommunicationParameter:
			return "PDO_COMMUNICATION_PARAMETER";
		case ObjectEntryDataType::PdoMapping:
			return "PDO_MAPPING";
		case ObjectEntryDataType::Identity:
			return "IDENTITY";
		case ObjectEntryDataType::SyncmgrSynchronization:
			return "SYNCMGR_SYNCHRONIZATION";
		case ObjectEntryDataType::UserStorageArea:
			return "USER_STORAGE_AREA";
		case ObjectEntryDataType::SamplerConfig:
			return "SAMPLER CONFIG";
		case ObjectEntryDataType::SamplerTimebase:
			return "SAMPLER TIMEBASE";
		case ObjectEntryDataType::PdiInput:
			return "PDI_INPUT";
		case ObjectEntryDataType::PdiOutput:
			return "PDI_OUTPUT";
		case ObjectEntryDataType::ClosedLoopControllerParameter:
			return "CLOSED_LOOP_CONTROLLER_PARAMETER";
		case ObjectEntryDataType::OpenLoopControllerParameter:
			return "OPEN_LOOP_CONTROLLER_PARAMETER";
		case ObjectEntryDataType::SsiEncoder:
			return "SSI ENCODER";
		case ObjectEntryDataType::N2:
			return "N2";
		case ObjectEntryDataType::N4:
			return "N4";
		case ObjectEntryDataType::V2:
			return "V2";
		case ObjectEntryDataType::X2:
			return "X2";
		case ObjectEntryDataType::X4:
			return "X4";
		case ObjectEntryDataType::InterpolationTimePeriod:
			return "INTERPOLATION_TIME_PERIOD";
		case ObjectEntryDataType::InterpolationDataConfiguration:
			return "INTERPOLATION_DATA_CONFIGURATION";
		case ObjectEntryDataType::VelocityAccelerationDeceleration:
			return "VELOCITY_ACCELERATION_DECELERATION";
		case ObjectEntryDataType::Simple:
			return "SIMPLE DATA TYPE";
		case ObjectEntryDataType::Complex:
			return "COMPLEX DATA TYPE";
		case ObjectEntryDataType::Invalid:
			return "INVALID DATA TYPE";
		default:
			return "Unknown DataType";
		}
	};

	/**
	 * @brief Convert std::string to ODDatatype if possible else return UNKNOWN_DATATYPE
	 *
	 * @param datatypeString
	 * @return ODDatatype
	 */
	static ObjectEntryDataType stringToObjectEntryDatatype(std::string dataTypeString) {
		dataTypeString = toUpperCase(dataTypeString);

		if (dataTypeString == "BOOLEAN") {
			return ObjectEntryDataType::Boolean;
		}
		if (dataTypeString == "INTEGER8") {
			return ObjectEntryDataType::Integer8;
		}
		if (dataTypeString == "INTEGER16") {
			return ObjectEntryDataType::Integer16;
		}
		if (dataTypeString == "INTEGER32") {
			return ObjectEntryDataType::Integer32;
		}
		if (dataTypeString == "UNSIGNED8") {
			return ObjectEntryDataType::Unsigned8;
		}
		if (dataTypeString == "UNSIGNED16") {
			return ObjectEntryDataType::Unsigned16;
		}
		if (dataTypeString == "UNSIGNED32") {
			return ObjectEntryDataType::Unsigned32;
		}
		if (dataTypeString == "REAL32") {
			return ObjectEntryDataType::Real32;
		}
		if (dataTypeString == "VISIBLE_STRING") {
			return ObjectEntryDataType::VisibleString;
		}
		if (dataTypeString == "OCTET_STRING") {
			return ObjectEntryDataType::OctetString;
		}
		if (dataTypeString == "UNICODE_STRING") {
			return ObjectEntryDataType::UnicodeString;
		}
		if (dataTypeString == "TIME_OF_DAY") {
			return ObjectEntryDataType::TimeDifference;
		}
		if (dataTypeString == "TIME_DIFFERENCE") {
			return ObjectEntryDataType::TimeDifference;
		}
		if (dataTypeString == "DOMAIN") {
			return ObjectEntryDataType::Domain;
		}
		// Complex types
		if (dataTypeString == "PDO_COMMUNICATION_PARAMETER") {
			return ObjectEntryDataType::PdoCommunicationParameter;
		}
		if (dataTypeString == "PDO_MAPPING") {
			return ObjectEntryDataType::PdoMapping;
		}
		if (dataTypeString == "IDENTITY") {
			return ObjectEntryDataType::Identity;
		}
		if (dataTypeString == "SYNCMGR_SYNCHRONIZATION") {
			return ObjectEntryDataType::SyncmgrSynchronization;
		}
		if (dataTypeString == "USER_STORAGE_AREA") {
			return ObjectEntryDataType::UserStorageArea;
		}
		if (dataTypeString == "SAMPLER CONFIG") {
			return ObjectEntryDataType::SamplerConfig;
		}
		if (dataTypeString == "SAMPLER TIMEBASE") {
			return ObjectEntryDataType::SamplerTimebase;
		}
		if (dataTypeString == "PDI_INPUT") {
			return ObjectEntryDataType::PdiInput;
		}
		if (dataTypeString == "PDI_OUTPUT") {
			return ObjectEntryDataType::PdiOutput;
		}
		if (dataTypeString == "CLOSED_LOOP_CONTROLLER_PARAMETER") {
			return ObjectEntryDataType::ClosedLoopControllerParameter;
		}
		if (dataTypeString == "OPEN_LOOP_CONTROLLER_PARAMETER") {
			return ObjectEntryDataType::OpenLoopControllerParameter;
		}
		if (dataTypeString == "SSI ENCODER") {
			return ObjectEntryDataType::SsiEncoder;
		}
		if (dataTypeString == "N2") {
			return ObjectEntryDataType::N2;
		}
		if (dataTypeString == "N4") {
			return ObjectEntryDataType::N4;
		}
		if (dataTypeString == "V2") {
			return ObjectEntryDataType::V2;
		}
		if (dataTypeString == "X2") {
			return ObjectEntryDataType::X2;
		}
		if (dataTypeString == "X4") {
			return ObjectEntryDataType::X4;
		}
		if (dataTypeString == "INTERPOLATION_TIME_PERIOD") {
			return ObjectEntryDataType::InterpolationTimePeriod;
		}
		if (dataTypeString == "INTERPOLATION_DATA_CONFIGURATION") {
			return ObjectEntryDataType::InterpolationDataConfiguration;
		}
		if (dataTypeString == "VELOCITY_ACCELERATION_DECELERATION") {
			return ObjectEntryDataType::VelocityAccelerationDeceleration;
		}
		return ObjectEntryDataType::Unknown;
	};
	
	static uint32_t objectEntryDataTypeBitLength(const ObjectEntryDataType dataType) {
		switch (dataType) {
		case ObjectEntryDataType::Boolean:
			return 8;
		case ObjectEntryDataType::Integer8:
			return 8;
		case ObjectEntryDataType::Integer16:
			return 16;
		case ObjectEntryDataType::Integer32:
			return 32;
		case ObjectEntryDataType::Unsigned8:
			return 8;
		case ObjectEntryDataType::Unsigned16:
			return 16;
		case ObjectEntryDataType::Unsigned32:
			return 32;
		case ObjectEntryDataType::Real32:
			return 32;
		case ObjectEntryDataType::TimeOfDay:
			return 0x30;
		case ObjectEntryDataType::TimeDifference:
			return 0x30;
		case ObjectEntryDataType::Integer24:
			return 24;
		case ObjectEntryDataType::Real64:
			return 64;
		case ObjectEntryDataType::Integer40:
			return 40;
		case ObjectEntryDataType::Integer48:
			return 48;
		case ObjectEntryDataType::Integer56:
			return 56;
		case ObjectEntryDataType::Integer64:
			return 64;
		case ObjectEntryDataType::Unsigned24:
			return 24;
		case ObjectEntryDataType::Unsigned40:
			return 24;
		case ObjectEntryDataType::Unsigned48:
			return 48;
		case ObjectEntryDataType::Unsigned56:
			return 56;
		case ObjectEntryDataType::Unsigned64:
			return 64;
		case ObjectEntryDataType::N2:
			return 16;
		case ObjectEntryDataType::N4:
			return 32;
		case ObjectEntryDataType::V2:
			return 16;
		case ObjectEntryDataType::X2:
			return 16;
		case ObjectEntryDataType::X4:
			return 32;
		default:
			return 0;
		}
	}

private:
	// Fix warning: https://adirmeier.de/Blog/ID_220
	static std::string toUpperCase(std::string stringToConvert) {
		std::transform(std::begin(stringToConvert), std::end(stringToConvert),
					   std::begin(stringToConvert),
					   [](int c) -> char { return static_cast<char>(::toupper(c)); });

		return stringToConvert;
	}

};

}