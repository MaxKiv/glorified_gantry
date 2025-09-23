#pragma once

#include <variant>
#include <string>
#include <vector>
#include <cstdint>

namespace nlc {
	
/**
 *
 * @brief CANOpen object codes
 *
 */
enum class ObjectCode {
	Null = 0x00,
	Deftype = 0x05,
	Defstruct = 0x06,
	Var = 0x07,
	Array = 0x08,
	Record = 0x09
};

/**
 *
 * @brief CANOpen object modification attribute
 *
 */
enum class ObjectModificationAttribute { Constant, Variable };

/**
 *
 * @brief CANOpen object SDO access attribute
 *
 */
enum class ObjectSdoAccessAttribute {
	ReadOnly = 1,
	WriteOnly = 2,
	ReadWrite = 3,
	NoAccess = 0
};

/**
 *
 * @brief CANOpen object PDO access attribute
 *
 */
enum class ObjectPdoAccessAttribute {
	Tx = 1,
	Rx = 2,
	TxRx = 3,
	No = 0,
};

/**
 *
 * @brief CANOpen object entry data types
 *
 */
enum class ObjectEntryDataType : uint16_t {
	Invalid = 0x0000,
	Boolean = 0x0001,
	Integer8 = 0x0002,
	Integer16 = 0x0003,
	Integer32 = 0x0004,
	Unsigned8 = 0x0005,
	Unsigned16 = 0x0006,
	Unsigned32 = 0x0007,
	Real32 = 0x0008,
	VisibleString = 0x0009,
	OctetString = 0x000A,
	UnicodeString = 0x000B,
	TimeOfDay = 0x000C,
	TimeDifference = 0x000D,	
	Domain = 0x000F,
	Integer24 = 0x0010,
	Real64 = 0x0011,
	Integer40 = 0x0012,
	Integer48 = 0x0013,
	Integer56 = 0x0014,
	Integer64 = 0x0015,
	Unsigned24 = 0x0016,
	Unsigned40 = 0x0018,
	Unsigned48 = 0x0019,
	Unsigned56 = 0x001A,
	Unsigned64 = 0x001B,
	PdoCommunicationParameter = 0x0020,
	PdoMapping = 0x0021,
	Identity = 0x0023,
	SyncmgrSynchronization = 0x0029,
	UserStorageArea = 0x0040,
	SamplerConfig = 0x0042,
	SamplerTimebase = 0x0043,
	PdiInput = 0x0045,
	PdiOutput = 0x0046,
	ClosedLoopControllerParameter = 0x0047,
	OpenLoopControllerParameter = 0x0048,
	SsiEncoder = 0x0049,
	N2 = 0x0071,
	N4 = 0x0072,
	V2 = 0x0073,
	X2 = 0x007B,
	X4 = 0x007C,
	InterpolationTimePeriod = 0x0080,
	InterpolationDataConfiguration = 0x0081,
	VelocityAccelerationDeceleration = 0x0082,
	Simple = 0x0777,
	Complex = 0x0FFF,
	Unknown = 0xFFFF
	// Currently, other types are not supported
};

/**
 *
 * @brief Nanotec object entry saveable types 
 *
 */
enum class ObjectSaveable {
	APPLICATION,
	COMMUNICATION,
	DRIVE,
	MISC_CONFIG,
	MODBUS_RTU,
	NO,
	TUNING,
	CUSTOMER,
	ETHERNET,
	CANOPEN,
	PROFIBUS,
	VERIFY1020,
	UNKNOWN_SAVEABLE_TYPE
};



} // namespace nlc