#pragma once

#include <string>
#include <vector>
#include <map>

#include "od_types.hpp"
#include "result.hpp"


namespace nlc {
/**
 *
 * @brief Class representing a object sub-entry
 *
 */
class ObjectSubEntry {
public:
	ObjectSubEntry() {
	}
	
	virtual ~ObjectSubEntry() {
	}

	virtual std::string getName() const {
		return std::string();
	}

	virtual uint8_t getSubIndex() const {
		return 0;
	}

	virtual nlc::ObjectEntryDataType getDataType() const {
		return nlc::ObjectEntryDataType::Invalid;
	}

	virtual nlc::ObjectSdoAccessAttribute getSdoAccess() const {
		return nlc::ObjectSdoAccessAttribute::NoAccess;
	}

	virtual nlc::ObjectPdoAccessAttribute getPdoAccess() const {
		return nlc::ObjectPdoAccessAttribute::No;
	}

	virtual uint32_t getBitLength() const {
		return 0;
	}

	virtual ResultInt getDefaultValueAsNumeric(std::string const &key) const {
		(void)key;
		return ResultInt(NlcErrorCode::InvalidArguments, "No default value");
	}

	virtual ResultString getDefaultValueAsString(std::string const &key) const {
		(void)key;
		return ResultString(NlcErrorCode::InvalidArguments, "No default value");
	}

	virtual std::map<std::string, std::string> getDefaultValues() const {
		return std::map<std::string, std::string>();
	}

	virtual ResultInt readNumber() const {
		return ResultInt(NlcErrorCode::ODDoesNotExist, "Invalid object");
	}

	virtual ResultString readString() const {
		return ResultString(NlcErrorCode::ODDoesNotExist, "Invalid object");
	}

	virtual ResultArrayByte readBytes() const {
		return ResultArrayByte(NlcErrorCode::ODDoesNotExist, "Invalid object");
	}

	virtual ResultVoid writeNumber(const int64_t value) const {
		(void)value;
		return ResultVoid(NlcErrorCode::ODDoesNotExist, "Invalid object");
	}

	virtual ResultVoid writeBytes(std::vector<uint8_t> const &data) const {
		(void)data;
		return ResultVoid(NlcErrorCode::ODDoesNotExist, "Invalid object");
	}
};

} // namespace nlc
