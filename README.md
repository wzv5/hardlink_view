使用 rust 实现 Windows explorer 自定义属性表的尝试。

涉及 COM 接口

- IShellExtInit
- IShellPropSheetExt
- IClassFactory
- IRegistrar (ATL)

用法

编译出 dll 后，使用 `regsvr32.exe` 注册。
注意，只有具有多个硬链接的文件才会显示。
