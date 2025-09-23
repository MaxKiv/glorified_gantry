#pragma once

#include "od_index.hpp"
#include "od_types.hpp"
#include "result.hpp"
#include "result_od_entry.hpp"

namespace nlc {
/**
* 
* @brief Class representing a object dictionary
* 
*/
class ObjectDictionary {
public:
	ObjectDictionary() {
	}

	virtual ~ObjectDictionary() {
	}

	virtual ResultString getXmlFileName() const {
		return ResultString(NlcErrorCode::ResourceNotFound, "File name not present.");
	}

	virtual ResultDeviceHandle getDeviceHandle() const {
		return ResultDeviceHandle("Dictionary is not assigned");
	}

	virtual ResultObjectSubEntry getObject(OdIndex const odIndex) {
		(void)odIndex;
		return ResultObjectSubEntry(NlcErrorCode::ODDoesNotExist, "Object doesn't exist");
	}

	virtual ResultObjectEntry getObjectEntry(uint16_t index) {
		(void)index;
		return ResultObjectEntry(NlcErrorCode::ODDoesNotExist, "Object doesn't exist");
	}

	virtual ResultInt readNumber(OdIndex const odIndex) {
		(void)odIndex;
		return ResultInt(NlcErrorCode::ODDoesNotExist, "Object doesn't exist");
	}

	virtual ResultString readString(OdIndex const odIndex) {
		(void)odIndex;
		return ResultString(NlcErrorCode::ODDoesNotExist, "Object doesn't exist");
	}

	virtual ResultArrayByte readBytes(OdIndex const odIndex) {
		(void)odIndex;
		return ResultArrayByte(NlcErrorCode::ODDoesNotExist, "Object doesn't exist");
	}

	virtual ResultVoid writeNumber(OdIndex const odIndex, const int64_t value) {
		(void)odIndex;
		(void)value;
		return ResultVoid(NlcErrorCode::ODDoesNotExist, "Object doesn't exist");
	}

	virtual ResultVoid writeBytes(OdIndex const odIndex,
									  std::vector<uint8_t> const &data) {
		(void)odIndex;
		(void)data;
		return ResultVoid(NlcErrorCode::ODDoesNotExist, "Object doesn't exist");
	}

	virtual ResultArrayInt readNumberArray(uint16_t const index) {
		(void)index;
		return ResultArrayInt(NlcErrorCode::ODDoesNotExist, "Object doesn't exist");
	}


};

} // namespace nlc