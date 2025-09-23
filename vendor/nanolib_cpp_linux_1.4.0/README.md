# Nanolib

This is the C++ version of NanoLib with an example application. <br>
The NanoLib offers an easy to use library to control Nanotec devices.

[www.nanotec.de](https://www.nanotec.de/)

## Example Application
### Overview and Structure
The CLI example application provides a menu interface where the user can execute
the different library functions. The menu offers the user the possibility to <br>
easily and quickly select and execute all functions supported by NanoLib. <br>
The menu entries are context based and will be enabled or disabled, depending on
the state.<br>
To enable all entries you have to:
1. Scan for hardware buses
2. Connect to a found harwdare bus
3. Scan for devices on the opened hardware bus
4. Successfully connect to a found device<br>
 
With this example application it is possible to:<br> 
- do a hardware bus scan
- open a found bus hardware (several hardware buses allowed)
- close an opened bus hardware
- scan for devices on opened hardware bus(es)
- connect to a found device (several devices allowed)
- disconnect from a connected device 
- read device informations
- update the firmware
- update the bootloader
- upload a NanoJ program
- run/stop a NanoJ program
- reboot a device
- set logging and logging callback parameters
- auto tune a motor (may require manual steps before)
- get a motor to rotate
- use the object dicationary interface for reads/writes
- sample data
- scan for Profinet devices
- etc.

The application menu and the supported NanoLib functionality is logically structered into several files:<br>
Files with \*_functions_example.* contain the implementations for the NanoLib interface functions.<br>
Files with \*_callback_example.* contain implementations for the various callbacks (scan, data and logging).<br>
Files with menu\_\*.\* contain the menu logic and code.<br>
Example.* is the main program, creating the menu and initializing all used parameters.<br>
Sampler_example.* contains the example implementation for sampler usage.<br>

### Windows
#### Prerequisites
- Install [Microsoft Visual Studio (Community) 2022](https://visualstudio.microsoft.com/de/vs/community/) or later
- Install [HMS - Ixxat VCI 4 driver](https://hmsnetworks.blob.core.windows.net/nlw/docs/default-source/products/ixxat/monitored/pc-interface-cards/vci-v4-0-1240-windows-11-10.zip?sfvrsn=2d1dfdd7_69) and connect CAN adapter (optional).
- Install [PEAK device driver and PCAN API](https://www.peak-system.com/quick/DrvSetup) and connect CAN adapter (optional).
- Connect all devices to your adapter(s) according to the user manual and power on the devices.

#### Compiling and running the example project
- Open the file ```Example.sln``` (below NanolibExample) with Visual Studio. The main file for the example is ```example.cpp```.
- Wait some seconds until the loading of the project is complete.
- Compile and run the example code.
  
#### Compiling and running the nanolib in your own project
##### Prerequisites
- If not already done, unpack all files and folders of the nanolib_cpp_win_#.#.#.zip file.
- Create a folder for your local project - Example: ```C:\LocalProject```

##### Create new project
- Open Visual Studio 2022, on the welcome screen click on "Create new project".
- You can now choose the type of the project. Choose "Console App - C++", click next.
- You now need to give the project a name (e.g. "NanolibTest") and set its location to your local project folder.
- Click on "Create"
- Select a configuration Release, 64-bit
- Copy the folders ```lib``` and ```inc``` from the extracted ```nanotec_nanolib``` folder into your local project folder. 
- Open the settings of the project
   1. In the compiler settings
      1. add the ```inc``` path to the "Additional Include Paths"
      2. and set the "C++ Language Standard" (under "Language") to "ISO C++17 (/std:c++17)"
   2. In the linker settings
      1. add the ```lib``` path of the dlls to "Additional Library Directories"
      2. add the file "nanolib.lib" to the "Additional Dependencies"
- Copy the following code into the main file and save it:
```C++
#include "accessor_factory.hpp"
#include <iostream>

int main()
{
	// DO NOT TRY TO delete THIS POINTER!
	nlc::NanoLibAccessor* nanolibAccessor = getNanoLibAccessor();
	nlc::ResultBusHwIds result = nanolibAccessor->listAvailableBusHardware();

	if (result.hasError())
	{
		std::cerr << "listAvailableBusHardware failed with error: " <<
			static_cast<int>(result.getErrorCode()) << " " << result.getError() << std::endl;
	}
	else
	{
		auto busHardwareIds(result.getResult());

		std::cout << "Available hardware buses and protocols:" << std::endl;

		for (const auto& busHardwareId : busHardwareIds)
		{
			std::cout << busHardwareId.getName()
				<< " protocol: " << busHardwareId.getProtocol() << std::endl;
		}
	}

	return 0;
}

```
- Compile everything
 In the compile output window, there should be no error but something like
```
========== Build: 1 succeeded, 0 failed, 0 up-to-date, 0 skipped ==========
```
- Before running the executable, copy the file ```nanolib.dll``` and all ```nanolibm_*.dll``` files to the same folder of the executable.
- Upon successful execution, the application will display a list of available hardware buses and communication protocols


### Linux
#### Prerequisites
- Install [HMS Ixxat ECI driver](https://hmsnetworks.blob.core.windows.net/nlw/docs/default-source/products/ixxat/monitored/pc-interface-cards/eci-linux.zip?sfvrsn=19eb48d7_53) and connect CAN adapter (optional).
- Download, build and install [PEAK device driver for linux and PCAN API](https://www.peak-system.com/quick/PCAN-Linux-Driver) and connect CAN adapter (optional).
- Connect all devices to your adapter(s) according to the user manual and power on the devices.
  
#### Compiling and running the example project
- Open a command prompt (e.g. bash) and change to extracted nanolib example directory. The main file for the example is ```src/example.cpp```.
- Build the example
```bash
  make all
```
- Install the example
 ```bash
  sudo make install
``` 
> **_Note:_** The following command from the make script will require sudo privileges:  ```sudo setcap cap_net_raw,cap_net_admin=eip $(OUT_PATH)/example``` \
> It is necessary to allow ethernet based bus scanning. See: https://linux.die.net/man/7/capabilities

- Execute the example
```bash
  ./bin/example
```

- Uninstall the Library if needed
```bash
  sudo make uninstall
```