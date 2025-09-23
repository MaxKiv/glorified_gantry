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
* @file   menu_utils.cpp
*
* @brief  Definition of CLI menu specific classes
*
* @date   29-10-2024
*
* @author Michael Milbradt
*
*/
#include "menu_utils.hpp"

namespace nlc {

Menu::Menu() { 
    this->default_func = nullptr;
}

Menu::Menu(const std::string &t, const vmi &vm, const f_type df)
    : title(t), menuItems(vm), default_func(df) {
}

bool Menu::eraseMenuItem(size_t index) {
    if (index < menuItems.size()) {
        menuItems.erase(menuItems.begin() + index);
        return true;
    }

    return false;
}

bool Menu::eraseAllMenuItems() {
    auto it = menuItems.begin();
    while (it != menuItems.end()) {
        it = menuItems.erase(it);
    }
    return true;
}

bool Menu::appendMenuItem(const MenuItem &menuItem) {
    menuItems.emplace_back(menuItem);
    return true;
}

bool Menu::insertMenuItem(size_t index, const MenuItem &menuItem) {
    if (index < menuItems.size()) {
        menuItems.insert(menuItems.begin() + index, menuItem);
        return true;
    }

    return false;
}

vector<DeviceId> Menu::getConnectableDeviceIds(const Context &ctx) {
    vector<DeviceId> result;

    // go through all found devices (by scan)
    for (auto scannedDeviceId : ctx.scannedDeviceIds) {
        bool alreadyConnected = false;
        // check with already connected devices
        for (auto connectedDeviceHandle : ctx.connectedDeviceHandles) {
            auto connectedDeviceId = ctx.nanolibAccessor->getDeviceId(connectedDeviceHandle).getResult();
            if (connectedDeviceId.equals(scannedDeviceId)) {
                alreadyConnected = true;
                break;
            }
        }

        if (!alreadyConnected) {
            result.push_back(scannedDeviceId);
        }
    }

    return result;
}

vector<BusHardwareId> Menu::getOpenableBusHwIds(const Context &ctx) {
    vector<BusHardwareId> result;

    // go through all found devices (by scan)
    for (auto scannedBusHw : ctx.scannedBusHardwareIds) {
        bool alreadyOpened = false;
        // check with already connected devices
        for (auto openBusHwId : ctx.openBusHardwareIds) {
            if (openBusHwId.equals(scannedBusHw)) {
                alreadyOpened = true;
                break;
            }
        }

        if (!alreadyOpened) {
            result.push_back(scannedBusHw);
        }
    }

    return result;
}

void Menu::setMenuItems(Menu &menu, Context &ctx) {
    if (menu.getTitle() == MAIN_MENU) {
         // check main menu items
        for (auto &mi : menu.menuItems) {
            if (mi.name == BUS_HARDWARE_MENU) {
                // always true since first possible action
                mi.isActivce = true;
            } else if (mi.name == DEVICE_MENU) {
                // active if bus hardware openend
                if (ctx.openBusHardwareIds.size() > 0) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            } else if ((mi.name == OD_INTERFACE_MENU) || 
                       (mi.name == SAMPLER_EXAMPLE_MENU) || 
                       (mi.name == MOTOR_EXAMPLE_MENU) || 
                       (mi.name == PROFINET_EXAMPLE_MI)) {
                // active of active device is set
                if (ctx.activeDevice.get() != 0) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            } else if (mi.name == LOGGING_MENU) {
                // always true, always possible
                mi.isActivce = true;
                
            } else {
                // do nothing
            } 
        }
    } else if (menu.getTitle() == BUS_HARDWARE_MENU) { 
        for (auto &mi : menu.menuItems) {
            if (mi.name == BUS_HARDWARE_SCAN_MI) {
                // always active
                mi.isActivce = true;
            } else if (mi.name == BUS_HARDWARE_OPEN_MI) {
                // active if we have bus hardware to open
                if (ctx.openableBusHardwareIds.size() > 0) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            } else if ((mi.name == BUS_HARDWARE_CLOSE_MI) || (mi.name == BUS_HARDWARE_CLOSE_ALL_MI)) {
                // active if we have opened bus hardware before
                if (ctx.openBusHardwareIds.size() > 0) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            } else {
                // do nothing
                // unknown menu entry
            }
        }
    } else if (menu.getTitle() == DEVICE_MENU) {
        for (auto &mi : menu.menuItems) {
            if (mi.name == DEVICE_SCAN_MI) {
                // activate if bus hardware is open
                if (ctx.openBusHardwareIds.size() > 0 ) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            } else if (mi.name == DEVICE_CONNECT_MENU) {
                // activate if devices are available after scan
                if (ctx.connectableDeviceIds.size() > 0) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            }  else if (mi.name == DEVICE_DISCONNECT_MENU) {
                // activate if device is connected
                if (ctx.connectedDeviceHandles.size() > 0) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            } else if (mi.name == DEVICE_SELECT_ACTIVE_MENU) {
                // activate if device is connected
                if (ctx.connectedDeviceHandles.size() > 0) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            } else if ((mi.name == DEVICE_INFORMATION_MENU) || 
                       (mi.name == DEVICE_REBOOT_MI) || 
                       (mi.name == DEVICE_UPDATE_FW_MI) || 
                       (mi.name == DEVICE_UPDATE_BL_MI) || 
                       (mi.name == DEVICE_UPLOAD_NANOJ_MI) || 
                       (mi.name == DEVICE_RUN_NANOJ_MI) || 
                       (mi.name == DEVICE_STOP_NANOJ_MI) ||
                       (mi.name == DEVICE_GET_ERROR_FIELD_MI) || 
                       (mi.name == DEVICE_RESTORE_ALL_DEFAULT_PARAMS_MI)) {
                // activate if active device is set
                if (ctx.activeDevice.get() != 0) {
                    mi.isActivce = true;
                } else {
                    mi.isActivce = false;
                }
            } else {
                // do nothing 
                // unknown menu option
            }
        }
    } else if ((menu.getTitle() == DEVICE_INFORMATION_MENU) || 
               (menu.getTitle() == OD_INTERFACE_MENU) || 
               (menu.getTitle() == SAMPLER_EXAMPLE_MENU) || 
               (menu.getTitle() == MOTOR_EXAMPLE_MENU)) {
          for (auto &mi : menu.menuItems) {
            // activate all menu entries if active device is selected
            if (ctx.activeDevice.get() != 0) {
                mi.isActivce = true;
            } else {
                mi.isActivce = false;
            }
        }
    } else if ((menu.getTitle() == LOG_LEVEL_MENU) || 
               (menu.getTitle() == LOGGING_MENU) || 
               (menu.getTitle() == LOG_CALLBACK_MENU)) {
        for (auto &mi : menu.menuItems) {
            // always active
            mi.isActivce = true;
        }
    } else if (menu.getTitle() == BUS_HARDWARE_OPEN_MI) {
        // dynamic menu
        // clear all menu items
        menu.eraseAllMenuItems();
        vector<BusHardwareId> openableBusHardwareIds = Menu::getOpenableBusHwIds(ctx);

        // re-build menu items, depending on found bus hardware and set defined funtion
        for (const auto openableBusHwId : openableBusHardwareIds) {
            MenuItem mi;
            mi.name = openableBusHwId.getProtocol() + " (" + openableBusHwId.getName() + ")";
            mi.func = menu.getDefaultFunction();
            // always active
            mi.isActivce = true;
            menu.appendMenuItem(mi);
        }
    } else if (menu.getTitle() == BUS_HARDWARE_CLOSE_MI) {
        // dynamic menu
        // clear all menu items
        menu.eraseAllMenuItems();
        vector<BusHardwareId> openBusHwIds = ctx.openBusHardwareIds;

        // re-build menu items, depending on found bus hardware and set defined funtion
        for (const auto openBusHwId : openBusHwIds) {
            MenuItem mi;
            mi.name = openBusHwId.getProtocol() + " (" + openBusHwId.getBusHardware() + ")";
            mi.func = menu.getDefaultFunction();
            // always active
            mi.isActivce = true;
            menu.appendMenuItem(mi);
        }
    } else if (menu.getTitle() == DEVICE_CONNECT_MENU) {
        // dynamic menu
        // clear all menu items
        menu.eraseAllMenuItems();
        vector<DeviceId> connectableDeviceIds = getConnectableDeviceIds(ctx);
        // re-build menu items, depending on found devices
        for (const auto connectableDeviceId : connectableDeviceIds) {
            MenuItem mi;
            mi.name = connectableDeviceId.getDescription()
                        + " [id: " + to_string(connectableDeviceId.getDeviceId())
                        + ", protocol: " + connectableDeviceId.getBusHardwareId().getProtocol()
                        + ", hw: " + connectableDeviceId.getBusHardwareId().getName() + "]";
            mi.func = menu.getDefaultFunction();
            // always active
            mi.isActivce = true;
            menu.appendMenuItem(mi);
        }
    } else if (menu.getTitle() == DEVICE_DISCONNECT_MENU) {
        // dynamic menu
        vector<DeviceId> openDeviceIds;
        // clear all menu items
        menu.eraseAllMenuItems();
        // find all opened device ids by device handle
        for (auto openDeviceHandle : ctx.connectedDeviceHandles) {
            ResultDeviceId openDeviceIdResult
                = ctx.nanolibAccessor->getDeviceId(openDeviceHandle);
            if (openDeviceIdResult.hasError()) {
                // ignore
                continue;
            }
            openDeviceIds.emplace_back(openDeviceIdResult.getResult());
        }

        // re-build menu items, depending on found devices
        for (const auto deviceId : openDeviceIds) {
            MenuItem mi;
            mi.name = deviceId.getDescription()
                        + " [id: " + to_string(deviceId.getDeviceId())
                        + ", protocol: " + deviceId.getBusHardwareId().getProtocol()
                        + ", hw: " + deviceId.getBusHardwareId().getName() + "]";
            mi.func = menu.getDefaultFunction();
            // always active
            mi.isActivce = true;
            menu.appendMenuItem(mi);
        }
    } else if (menu.getTitle() == DEVICE_SELECT_ACTIVE_MENU) {
        // dynamic menu
        // clear all menu items
        menu.eraseAllMenuItems();
        vector<DeviceHandle> connectedDeviceHandles = ctx.connectedDeviceHandles;
        // re-build menu items, depending on found devices
        for (const auto connectedDeviceHandle : connectedDeviceHandles) {
            ResultDeviceId deviceIdResult = ctx.nanolibAccessor->getDeviceId(connectedDeviceHandle);
            if (deviceIdResult.hasError()) {
                // ignore
                continue;
            }
            DeviceId deviceId = deviceIdResult.getResult();
            MenuItem mi;
            mi.name = deviceId.getDescription()
                        + " [id: " + to_string(deviceId.getDeviceId())
                        + ", protocol: " + deviceId.getBusHardwareId().getProtocol()
                        + ", hw: " + deviceId.getBusHardwareId().getName() + "]";
            mi.func = menu.getDefaultFunction();
            // always active
            mi.isActivce = true;
            menu.appendMenuItem(mi);
        }
    } else {
        // do nothing
        // unknown menu
    }
}

string Menu::getActiveDeviceString(Context &ctx) {
    ostringstream result;
    result <<  "Active device    : " << ctx.dark_gray << "None" << ctx.def << endl;
    if (ctx.activeDevice.get() == 0) {
        return result.str();
    }

    result.str(std::string());
    DeviceId activeDevice = ctx.nanolibAccessor->getDeviceId(ctx.activeDevice).getResult();
    result << "Active device    : " << ctx.light_green << activeDevice.getDescription() << " [id: " << to_string(activeDevice.getDeviceId())
                            << ", protocol: " << activeDevice.getBusHardwareId().getProtocol()
                            << ", hw: " << activeDevice.getBusHardwareId().getName() << "]" << ctx.def << endl;
    
    return result.str();
}

string Menu::getFoundBusHwString(Context &ctx) {
    ostringstream result;
    result << "Bus HW found     : " << ctx.dark_gray <<"None (not scanned?)" << ctx.def << endl;

    if (ctx.scannedBusHardwareIds.empty()) {
        return result.str();
    }

    result.str(std::string());
    result << "Bus HW found     : " << ctx.light_green << to_string(ctx.scannedBusHardwareIds.size()) << ctx.def << endl;
    return result.str();
}

string Menu::getOpenedBusHwIdString(Context &ctx) {
    ostringstream result;
    result << "Open Bus HW      : " << ctx.dark_gray << "None" << ctx.def << endl;

    if (ctx.openBusHardwareIds.empty()) {
        return result.str();
    }

    bool firstItem = true;
    result.str(std::string());
    result << "Open Bus HW      : ";
    for (auto openBusHardwareId : ctx.openBusHardwareIds) {
        if (firstItem) {
            result << ctx.light_green << openBusHardwareId.getProtocol() << " (" << openBusHardwareId.getName() << ")" << ctx.def;
            firstItem = false;
        } else {
            result << ", " << ctx.light_green << openBusHardwareId.getProtocol() << " (" << openBusHardwareId.getName() << ")" << ctx.def;
        }
        
    }
    
    result << endl;
    return result.str();
}

string Menu::getScannedDeviceIdsString(Context &ctx) {
    ostringstream result;
    result << "Device(s) found  : " << ctx.dark_gray << "None (not scanned?)" << ctx.def << endl;

    if (ctx.scannedDeviceIds.empty()) {
        return result.str();
    }
    result.str(std::string());
    result << "Device(s) found  : " << ctx.light_green << to_string(ctx.scannedDeviceIds.size()) << ctx.def << endl;
    return result.str();
}

string Menu::getConnectedDevicesString(Context &ctx) {
    ostringstream result;
    result << "Connected devices: " << ctx.dark_gray << "None" << ctx.def << endl;

    if (ctx.connectedDeviceHandles.empty()) {
        return result.str();
    }

    bool firstItem = true;
    result.str(std::string());
    result << "Connected devices: ";

    for (auto connectedDeviceHandle : ctx.connectedDeviceHandles) {
        ResultDeviceId resultDeviceId = ctx.nanolibAccessor->getDeviceId(connectedDeviceHandle);
        if (resultDeviceId.hasError()) {
            // don't display
            continue;
        }
        DeviceId connectedDeviceId = resultDeviceId.getResult();
        if (firstItem) {
            result << ctx.light_green << connectedDeviceId.getDescription()
                            << " [id: " + to_string(connectedDeviceId.getDeviceId())
                            << ", protocol: " << connectedDeviceId.getBusHardwareId().getProtocol()
                            << ", hw: " << connectedDeviceId.getBusHardwareId().getName() << "]" << ctx.def;
            firstItem = false;
        } else {
            result << ", " << ctx.light_green << connectedDeviceId.getDescription()
                           << " [id: " + to_string(connectedDeviceId.getDeviceId())
                           << ", protocol: " << connectedDeviceId.getBusHardwareId().getProtocol()
                           << ", hw: " << connectedDeviceId.getBusHardwareId().getName() << "]" << ctx.def;
        }
    }

    result << endl;

    return result.str();
}

string Menu::getCallbackLoggingString(Context &ctx) {
    ostringstream result;
    result << "Callback Logging : " << "Off" << endl;

    if (!ctx.loggingCallbackActive) {
        return result.str();
    }

    result.str(std::string());
    result << "Callback Logging : " << ctx.light_green << "On" << ctx.def << " (" << nlc::LogModuleConverter::toString(ctx.currentLogModule) << ")" << endl;
    return result.str();
}

string Menu::getObjectDictionaryString(Context &ctx) {
    ostringstream result;
    result << "Object dictionary: " << ctx.dark_gray << "Fallback" << " (not assigned)" << ctx.def << endl;

    if (ctx.activeDevice.get() == 0) {
        return result.str();
    }

    ResultObjectDictionary resultObjectDictionary = ctx.nanolibAccessor->getAssignedObjectDictionary(ctx.activeDevice);
    if (resultObjectDictionary.hasError()) {
        return result.str();
    }

    ObjectDictionary &objectDictionary = const_cast<ObjectDictionary&>(resultObjectDictionary.getResult());

    if (objectDictionary.getXmlFileName().getResult().empty()) {
        return result.str();
    }

    result.str(std::string());
    result << "Object dictionary: " << ctx.light_green << "Assigned" << ctx.def << endl;
    
    return result.str();
}

string Menu::printInfo(Context &ctx) const noexcept {
    // clear screen, return value not needed
#ifdef _WIN32
    int result = system("CLS");
#else
    int result = system("clear");
#endif
    (void)result;
    ostringstream oss;
    
    oss << getActiveDeviceString(ctx);
    oss << getFoundBusHwString(ctx);
    oss << getOpenedBusHwIdString(ctx);
    oss << getScannedDeviceIdsString(ctx);
    oss << getConnectedDevicesString(ctx);
    oss << getCallbackLoggingString(ctx);
    oss << getObjectDictionaryString(ctx);
    oss << "Log level        : " << LogLevelConverter::toString(ctx.currentLogLevel) << endl;
    // coloring is done in handleErrorMessage
    oss << ctx.errorText << endl; 
    // clear text
    ctx.errorText = "";
    return oss.str();
}

size_t Menu::showMenu(Menu &currentMenu, Context &ctx) {
    // dynamic part (for some menus)
    setMenuItems(currentMenu, ctx);
    // static part
    ostringstream oss;
    const auto numberOfMenuItems = currentMenu.menuItems.size();
    // if true, stop at the end of execution of the selected option
    // until return is pressed by the user
    if (ctx.waitForUserConfirmation) {
        cout << "Press enter to continue! " << endl;
        cin.get();
    }
    ctx.waitForUserConfirmation = false;
    // create the user information part
    oss << currentMenu.printInfo(ctx);
    // create the menu header
    oss << "---------------------------------------------------------------------------" << endl;
    oss << " " << currentMenu.getTitle() << endl;
    oss << "---------------------------------------------------------------------------" << endl;

    // create the menu items (options)
    for (size_t i = 1U; i <= numberOfMenuItems; ++i)
        if (currentMenu.menuItems[i-1].isActivce) {
            oss << ctx.def << (((numberOfMenuItems > 9) && (i < 10)) ? " " : "") << i << ") " << currentMenu.menuItems[i-1].name << '\n';
        } else {
            oss << ctx.dark_gray << (((numberOfMenuItems > 9) && (i < 10)) ? " " : "") << i << ") " << currentMenu.menuItems[i-1].name << ctx.def << '\n';
        }
        
    // create back (sub-menu) or exit option (main menu)
    if (currentMenu.getTitle() == MAIN_MENU) {
       oss << endl << ((numberOfMenuItems > 9) ? " " : "") << "0) " <<  "Exit program\n\nEnter menu option number";
    } else {
        oss << endl << ((numberOfMenuItems > 9) ? " " : "") << "0) " << "Back\n\nEnter menu option number";
    }

    // bring created output to screen and wait for user input
    return getnum<size_t>(oss.str(), 0, numberOfMenuItems);
}

void Menu::menu(Menu &menu, Context &ctx) {
    // clear screen, result not needed
#ifdef _WIN32
    int result = system("CLS");
#else
    int result = system("clear");
#endif
    (void)result;

    ctx.waitForUserConfirmation = false;
    for (size_t opt = 0U; (opt = showMenu(menu, ctx)) > 0;) {

        if ((opt == std::numeric_limits<size_t>::max()) || menu.menuItems.at(opt-1).isActivce == false) {
            ostringstream oss;
            oss << ctx.light_yellow << "Invalid option" << ctx.def;
            ctx.errorText = oss.str();
        } else {
            ctx.errorText = "";

            // store selected option to context
            ctx.selectedOption = opt;

            if (const auto &mi = menu.menuItems[opt - 1];
                holds_alternative<f_type>(mi.func)) {
                get<f_type>(mi.func)(ctx);
            } else {
                Menu::menu(*get<Menu *>(mi.func), ctx);
            }
        }
    }
}
} // namespace nlc
