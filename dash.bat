SET SCRIPTPATH=%~dp0
call "%SCRIPTPATH%dashboard\.venv\Scripts\activate.bat"
python "%SCRIPTPATH%dashboard\main.py"
if %errorlevel% neq 0 pause