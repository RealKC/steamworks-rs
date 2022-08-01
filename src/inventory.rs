use std::ptr;

use super::*;
use crate::error::SteamError;

#[doc(alias = "ISteamInventory")]
pub struct Inventory<Manager> {
    pub(crate) inventory: *mut sys::ISteamInventory,
    pub(crate) inner: Arc<Inner<Manager>>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[doc(alias = "SteamInventoryResult_t")]
pub struct InventoryResultHandle(sys::SteamInventoryResult_t);

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[doc(alias = "SteamItemDef_t")]
pub struct InventoryItemDefinition(sys::SteamItemDef_t);

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[doc(alias = "SteamInventoryFullUpdate_t")]
pub struct InventoryFullUpdate {
    pub handle: InventoryResultHandle,
}

unsafe impl Callback for InventoryFullUpdate {
    const ID: i32 = sys::SteamInventoryFullUpdate_t_k_iCallback as _;

    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryFullUpdate_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        debug_assert!(!raw.is_null());

        let val = &*(raw as *mut sys::SteamInventoryFullUpdate_t);

        Self {
            handle: InventoryResultHandle(val.m_handle),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[doc(alias = "SteamInventoryResultReady_t")]
pub struct InventoryResultReady {
    pub handle: InventoryResultHandle,
    pub result: SteamError,
}

unsafe impl Callback for InventoryResultReady {
    const ID: i32 = sys::SubmitPlayerResultResultCallback_t_k_iCallback as _;

    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryResultReady_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        debug_assert!(!raw.is_null());

        let val = &*(raw as *mut sys::SteamInventoryResultReady_t);

        Self {
            handle: InventoryResultHandle(val.m_handle),
            result: SteamError::from(val.m_result),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[doc(alias = "SteamInventoryDefinitionUpdate_t")]
pub struct InventoryDefinitionUpdate;

unsafe impl Callback for InventoryDefinitionUpdate {
    const ID: i32 = sys::SteamInventoryDefinitionUpdate_t_k_iCallback as _;

    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryDefinitionUpdate_t>() as _;

    unsafe fn from_raw(_raw: *mut c_void) -> Self {
        Self
    }
}

impl<Manager> Inventory<Manager> {
    #[doc(alias = "GrantPromoItems")]
    pub fn grant_promo_items(&self) -> SResult<(bool, InventoryResultHandle)> {
        let mut id = 0;
        let result =
            unsafe { sys::SteamAPI_ISteamInventory_GrantPromoItems(self.inventory, &mut id) };

        Ok((result, InventoryResultHandle(id)))
    }

    #[doc(alias = "GetAllItems")]
    pub fn get_all_items(&self) -> SResult<(bool, InventoryResultHandle)> {
        let mut id = 0;
        let result = unsafe { sys::SteamAPI_ISteamInventory_GetAllItems(self.inventory, &mut id) };

        Ok((result, InventoryResultHandle(id)))
    }

    #[doc(alias = "DestroyResult")]
    pub fn destroy_result(&self, result: InventoryResultHandle) {
        unsafe {
            sys::SteamAPI_ISteamInventory_DestroyResult(self.inventory, result.0);
        }
    }

    #[doc(alias = "CheckResultSteamID")]
    pub fn check_steam_id(&self, result: InventoryResultHandle, steam_id: SteamId) -> bool {
        unsafe {
            sys::SteamAPI_ISteamInventory_CheckResultSteamID(self.inventory, result.0, steam_id.0)
        }
    }

    #[doc(alias = "LoadItemDefinitions")]
    pub fn load_item_definitions(&self) {
        unsafe {
            sys::SteamAPI_ISteamInventory_LoadItemDefinitions(self.inventory);
        }
    }

    #[doc(alias = "GetItemDefinitionsIDs")]
    pub fn get_item_definitions(&self) -> SResult<Vec<InventoryItemDefinition>> {
        let mut item_definition_count = 0;

        let items_were_loaded = unsafe {
            sys::SteamAPI_ISteamInventory_GetItemDefinitionIDs(
                self.inventory,
                ptr::null_mut(),
                &mut item_definition_count,
            )
        };

        if !items_were_loaded {
            return Err(SteamError::ItemDefinitionsNotLoaded);
        }

        let mut definitions = Vec::with_capacity(item_definition_count as usize);

        let items_were_loaded = unsafe {
            sys::SteamAPI_ISteamInventory_GetItemDefinitionIDs(
                self.inventory,
                definitions.as_mut_ptr(),
                &mut item_definition_count,
            )
        };
        debug_assert!(
            items_were_loaded,
            "The item definitions should've stayed loaded from the previous call"
        );

        Ok(definitions
            .into_iter()
            .map(InventoryItemDefinition)
            .collect())
    }

    // pub fn get_all_item_definition_properties(&self) -> SResult<Vec<String>> {
    //     let mut buf_size = 0;
    //     unsafe { sys::SteamAPI_ISteamInventory_GetItemDefinitionProperty(self.inventory, iDefinition, pchPropertyName, pchValueBuffer, punValueBufferSizeOut)}

    // }

    pub fn get_item_definition_property(
        &self,
        item_definition: InventoryItemDefinition,
        name: &str,
    ) -> SResult<String> {
        let name = CString::new(name).unwrap();
        let mut buf_size = 0;

        unsafe {
            sys::SteamAPI_ISteamInventory_GetItemDefinitionProperty(
                self.inventory,
                item_definition.0,
                name.as_ptr(),
                ptr::null_mut(),
                &mut buf_size,
            )
        };

        let mut value = Vec::with_capacity(buf_size as usize);

        unsafe {
            sys::SteamAPI_ISteamInventory_GetItemDefinitionProperty(
                self.inventory,
                item_definition.0,
                name.as_ptr(),
                value.as_mut_ptr(),
                &mut buf_size,
            )
        };

        Ok(
            CStr::from_bytes_with_nul(&value.into_iter().map(|ch| ch as u8).collect::<Vec<_>>())
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
        )
    }

    pub fn inventory_result_ready_callback(
        &self,
        callback: impl Fn(InventoryResultReady) + Send + 'static,
    ) {
        unsafe { register_callback(&self.inner, callback) };
    }

    pub fn inventory_full_update_callback(
        &self,
        callback: impl Fn(InventoryFullUpdate) + Send + 'static,
    ) {
        unsafe { register_callback(&self.inner, callback) };
    }
}
