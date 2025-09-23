#pragma once

#include <string>

#include "od_types.hpp"
#include "od_sub_entry.hpp"

namespace nlc {

/**
 *
 * @brief Class representing a object entry
 *
 */
class ObjectEntry {
public:
	ObjectEntry() {
	}

	virtual ~ObjectEntry() {
	}

	virtual std::string getName() const {
		return std::string();
	}

	virtual bool getPrivate() const {
		return true;
	}

	virtual uint16_t getIndex() const {
		return 0;
	}

	virtual nlc::ObjectEntryDataType getDataType() const {
		return nlc::ObjectEntryDataType::Invalid;
	}

	virtual nlc::ObjectCode getObjectCode() const {
		return nlc::ObjectCode::Null;
	}

	virtual nlc::ObjectSaveable getObjectSaveable() const {
		return nlc::ObjectSaveable::UNKNOWN_SAVEABLE_TYPE;
	}

	virtual uint8_t getMaxSubIndex() const {
		return 0;
	};

	virtual nlc::ObjectSubEntry &getSubEntry(uint8_t subIndex) {
		(void)subIndex;
		return invalidObject;
	}

protected:
	inline static nlc::ObjectSubEntry invalidObject; 
};

} // namespace nlc
