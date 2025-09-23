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
* @file   data_transfer_callback_example.hpp
*
* @brief  Declaration of data transfer callback class
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
#include "nlc_callback.hpp"
#include "nlc_constants.hpp"

using namespace nlc;
using namespace std;

namespace nlc {

/// @brief Implementation class of NlcDataTransferCallback, handling the data transfer callback
class DataTransferCallbackExample : public NlcDataTransferCallback {

	/// @brief Gets called during data transfer
	/// @param info - state of data transfer
	/// @param data - progress of data transfer (0-100)
	/// @return - returns ResultVoid with error if error occured
	ResultVoid callback(DataTransferInfo info,
									  int32_t data) override;
};

} // namespace nlc