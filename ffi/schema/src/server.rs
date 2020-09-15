use crate::common::CommonDefinitions;

use oo_bindgen::callback::InterfaceHandle;
use oo_bindgen::class::{ClassDeclarationHandle, ClassHandle};
use oo_bindgen::native_function::{NativeFunctionHandle, ReturnType, Type};
use oo_bindgen::{BindingError, LibraryBuilder};

pub(crate) fn build(
    lib: &mut LibraryBuilder,
    common: &CommonDefinitions,
) -> Result<(), BindingError> {
    let _server = build_server(lib, common)?;
    Ok(())
}

pub(crate) fn build_server(
    lib: &mut LibraryBuilder,
    common: &CommonDefinitions,
) -> Result<ClassHandle, BindingError> {
    let handler_map = build_handler_map(lib, common)?;

    let server_handle = lib.declare_class("ServerHandle")?;

    let create_fn = lib
        .declare_native_function("create_tcp_server")?
        .param(
            "runtime",
            Type::ClassRef(common.runtime_handle.declaration.clone()),
            "runtime on which to spawn the server",
        )?
        .param("address", Type::String, "IPv4 or IPv6 host/port string")?
        .param(
            "endpoints",
            Type::ClassRef(handler_map.declaration.clone()),
            "map of endpoints which is emptied upon passing to this function",
        )?
        .return_type(ReturnType::Type(
            Type::ClassRef(server_handle.clone()),
            "handle to the server".into(),
        ))?
        .doc("Launch a TCP server to handle")?
        .build()?;

    let destroy_fn = lib
        .declare_native_function("destroy_server")?
        .param(
            "server",
            Type::ClassRef(server_handle.clone()),
            "handle of the server to destroy",
        )?
        .return_type(ReturnType::void())?
        .doc("destroy a running server via its handle")?
        .build()?;

    lib.define_class(&server_handle)?
        .constructor(&create_fn)?
        .destructor(&destroy_fn)?
        .doc("Server handle, the server remains alive until this reference is destroyed")?
        .build()
}

pub(crate) fn build_add_fn(
    lib: &mut LibraryBuilder,
    db: &ClassDeclarationHandle,
    snake_name: &str,
    value_type: Type,
) -> Result<NativeFunctionHandle, BindingError> {
    let spaced_name = snake_name.replace("_", " ");

    lib.declare_native_function(&format!("database_add_{}", snake_name))?
        .param(
            "database",
            Type::ClassRef(db.clone()),
            "database to manipulate",
        )?
        .param(
            "index",
            Type::Uint16,
            format!("address of the {}", spaced_name).as_str(),
        )?
        .param(
            "value",
            value_type,
            format!("initial value of the {}", spaced_name).as_str(),
        )?
        .return_type(ReturnType::Type(
            Type::Bool,
            "true if the value is new, false otherwise".into(),
        ))?
        .doc(format!("add a new {} to the database", spaced_name).as_str())?
        .build()
}

pub(crate) fn build_delete_fn(
    lib: &mut LibraryBuilder,
    db: &ClassDeclarationHandle,
    snake_name: &str,
) -> Result<NativeFunctionHandle, BindingError> {
    let spaced_name = snake_name.replace("_", " ");

    lib.declare_native_function(&format!("database_delete_{}", snake_name))?
        .param(
            "database",
            Type::ClassRef(db.clone()),
            "database to manipulate",
        )?
        .param(
            "index",
            Type::Uint16,
            format!("address of the {}", spaced_name).as_str(),
        )?
        .return_type(ReturnType::Type(
            Type::Bool,
            "true if the address existed and was removed, false otherwise".into(),
        ))?
        .doc(format!("remove a {} address from the database", spaced_name).as_str())?
        .build()
}

pub(crate) fn build_update_fn(
    lib: &mut LibraryBuilder,
    db: &ClassDeclarationHandle,
    snake_name: &str,
    value_type: Type,
) -> Result<NativeFunctionHandle, BindingError> {
    let spaced_name = snake_name.replace("_", " ");

    lib.declare_native_function(&format!("database_update_{}", snake_name))?
        .param(
            "database",
            Type::ClassRef(db.clone()),
            "database to manipulate",
        )?
        .param(
            "index",
            Type::Uint16,
            format!("address of the {}", spaced_name).as_str(),
        )?
        .param(
            "value",
            value_type,
            format!("new value of the {}", spaced_name).as_str(),
        )?
        .return_type(ReturnType::Type(
            Type::Bool,
            "true if the address is defined, false otherwise".into(),
        ))?
        .doc(
            format!(
                "update the current value of a {} in the database",
                spaced_name
            )
            .as_str(),
        )?
        .build()
}

pub(crate) fn build_database_class(lib: &mut LibraryBuilder) -> Result<ClassHandle, BindingError> {
    let database = lib.declare_class("Database")?;

    let add_coil_fn = build_add_fn(lib, &database, "coil", Type::Bool)?;
    let add_discrete_input_fn = build_add_fn(lib, &database, "discrete_input", Type::Bool)?;
    let add_holding_register_fn = build_add_fn(lib, &database, "holding_register", Type::Uint16)?;
    let add_input_register_fn = build_add_fn(lib, &database, "input_register", Type::Uint16)?;

    let update_coil_fn = build_update_fn(lib, &database, "coil", Type::Bool)?;
    let update_discrete_input_fn = build_update_fn(lib, &database, "discrete_input", Type::Bool)?;
    let update_holding_register_fn =
        build_update_fn(lib, &database, "holding_register", Type::Uint16)?;
    let update_input_register_fn = build_update_fn(lib, &database, "input_register", Type::Uint16)?;

    let delete_coil_fn = build_delete_fn(lib, &database, "coil")?;
    let delete_discrete_input_fn = build_delete_fn(lib, &database, "discrete_input")?;
    let delete_holding_register_fn = build_delete_fn(lib, &database, "holding_register")?;
    let delete_input_register_fn = build_delete_fn(lib, &database, "input_register")?;

    lib.define_class(&database)?
        // add methods
        .method("add_coil", &add_coil_fn)?
        .method("add_discrete_input", &add_discrete_input_fn)?
        .method("add_holding_register", &add_holding_register_fn)?
        .method("add_input_register", &add_input_register_fn)?
        // update methods
        .method("update_coil", &update_coil_fn)?
        .method("update_discrete_input", &update_discrete_input_fn)?
        .method("update_holding_register", &update_holding_register_fn)?
        .method("update_input_register", &update_input_register_fn)?
        // delete methods
        .method("delete_coil", &delete_coil_fn)?
        .method("delete_discrete_input", &delete_discrete_input_fn)?
        .method("delete_holding_register", &delete_holding_register_fn)?
        .method("delete_input_register", &delete_input_register_fn)?
        // docs
        .doc("Class used to add, remove, and update values")?
        .build()
}

pub(crate) fn build_handler_map(
    lib: &mut LibraryBuilder,
    common: &CommonDefinitions,
) -> Result<ClassHandle, BindingError> {
    let request_handler = build_request_handler_interface(lib, common)?;

    let database = build_database_class(lib)?;

    let device_map = lib.declare_class("DeviceMap")?;

    let create_map = lib
        .declare_native_function("create_device_map")?
        .return_type(ReturnType::Type(
            Type::ClassRef(device_map.clone()),
            "Device map instance".into(),
        ))?
        .doc("Create a device map that will be used to bind devices to a server endpoint")?
        .build()?;

    let destroy_map = lib
        .declare_native_function("destroy_device_map")?
        .param(
            "map",
            Type::ClassRef(device_map.clone()),
            "value to destroy",
        )?
        .return_type(ReturnType::void())?
        .doc("Destroy a previously created device map")?
        .build()?;

    let map_add_endpoint = lib
        .declare_native_function("map_add_endpoint")?
        .param(
            "map",
            Type::ClassRef(device_map.clone()),
            "map to which the endpoint will be added",
        )?
        .param("unit_id", Type::Uint8, "Unit id of the endpoint")?
        .param(
            "handler",
            Type::Interface(request_handler),
            "callback interface for handling read and write operations for this device",
        )?
        .return_type(ReturnType::Type(
            Type::ClassRef(database.declaration.clone()),
            "Pointer to the database instance for this endpoint, or NULL if it could not be created b/c of a duplicate unit id".into(),
        ))?
        .doc("add an endpoint to the map")?
        .build()?;

    lib.define_class(&device_map)?
        .constructor(&create_map)?
        .destructor(&destroy_map)?
        .method("add_endpoint", &map_add_endpoint)?
        .doc("Maps endpoint handlers to Modbus address")?
        .build()
}

pub(crate) fn build_request_handler_interface(
    lib: &mut LibraryBuilder,
    common: &CommonDefinitions,
) -> Result<InterfaceHandle, BindingError> {
    let write_result = lib.declare_native_struct("WriteResult")?;
    let write_result = lib
        .define_native_struct(&write_result)?
        .add("success", Type::Bool, "true if the operation was successful, false otherwise. Error details found in the exception field.")?
        .add("exception", Type::Enum(common.exception.clone()), "exception enumeration. If undefined, look at the raw value")?
        .add("raw_exception", Type::Uint8, "Raw exception value when 'exception' field is Undefined")?
        .doc("Result struct describing if an operation was successful or not. Exception codes are returned to the client")?
        .build()?;

    lib.define_interface(
        "RequestHandler",
        "Interface used to handle read and write requests received from the client",
    )?
    // --- write single coil ---
    .callback(
        "write_single_coil",
        "write a single coil received from the client",
    )?
    .param("value", Type::Bool, "Value of the coil to write")?
    .param("index", Type::Uint16, "Index of the coil")?
    .return_type(ReturnType::Type(
        Type::Struct(write_result.clone()),
        "struct describing the result of the operation".into(),
    ))?
    .build()?
    // --- write single register ---
    .callback(
        "write_single_register",
        "write a single coil received from the client",
    )?
    .param("value", Type::Uint16, "Value of the register to write")?
    .param("index", Type::Uint16, "Index of the register")?
    .return_type(ReturnType::Type(
        Type::Struct(write_result.clone()),
        "struct describing the result of the operation".into(),
    ))?
    .build()?
    // --- write multiple coils ---
    .callback(
        "write_multiple_coils",
        "write multiple coils received from the client",
    )?
    .param("start", Type::Uint16, "starting address")?
    .param(
        "it",
        Type::Iterator(common.bit_iterator.clone()),
        "iterator over coil values",
    )?
    .return_type(ReturnType::Type(
        Type::Struct(write_result.clone()),
        "struct describing the result of the operation".into(),
    ))?
    .build()?
    // --- write multiple registers ---
    .callback(
        "write_multiple_registers",
        "write multiple registers received from the client",
    )?
    .param("start", Type::Uint16, "starting address")?
    .param(
        "it",
        Type::Iterator(common.register_iterator.clone()),
        "iterator over register values",
    )?
    .return_type(ReturnType::Type(
        Type::Struct(write_result),
        "struct describing the result of the operation".into(),
    ))?
    .build()?
    // -------------------------------
    .destroy_callback("destroy")?
    .build()
}
