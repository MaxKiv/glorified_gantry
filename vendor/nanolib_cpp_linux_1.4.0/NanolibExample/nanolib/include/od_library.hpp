#pragma once

#include "result_od.hpp"

namespace nlc {
/**
 *
 * @brief Class representing a object dictionary library
 *
 */
class OdLibrary {
public:
	OdLibrary() {
	}

	virtual ~OdLibrary() {
	}

	virtual uint32_t getObjectDictionaryCount() const = 0;

	virtual ResultObjectDictionary getObjectDictionary(uint32_t odIndex) = 0;

	virtual ResultObjectDictionary
	addObjectDictionaryFromFile(std::string const &absoluteXmlFilePath)
		= 0;

	virtual ResultObjectDictionary addObjectDictionary(std::vector<uint8_t> const &odXmlData, const std::string& xmlFilePath = std::string()) = 0;

};

}
