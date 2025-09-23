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
* @file   data_transfer_callback_example.cpp
*
* @brief  Definition of data transfer callback class
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "data_transfer_callback_example.hpp"

namespace nlc {
    
ResultVoid DataTransferCallbackExample::callback(DataTransferInfo info,
									  int32_t data) {
    (void)data;
    
    switch (info) {
        case DataTransferInfo::Init:
            // do nothing
            break;
        case DataTransferInfo::FileOpen:
            cout << "Transfer started ..." << endl;
            break;
        case DataTransferInfo::Finished:
            cout << endl << "Transfer finished ..." << endl;
            break;
        case DataTransferInfo::Progress:
            if ((data & 1) == 0) // data holds transfer progress
            {
                cout << ".";
            }
            break;
        case DataTransferInfo::Reboot:
            cout << "Rebooting ..." << endl;
            break;
    }

    return ResultVoid();
}

} // namespace nlc