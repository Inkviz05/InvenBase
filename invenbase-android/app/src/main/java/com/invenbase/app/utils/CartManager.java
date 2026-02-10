package com.invenbase.app.utils;

import android.content.Context;
import android.content.SharedPreferences;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
import com.invenbase.app.Config;
import com.invenbase.app.models.CartItem;
import com.invenbase.app.models.Equipment;

import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.List;

public class CartManager {
    private static final String PREF_CART = "booking_cart";
    private final SharedPreferences prefs;
    private final Gson gson;

    public CartManager(Context context) {
        prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        gson = new Gson();
    }

    public List<CartItem> getItems() {
        String json = prefs.getString(PREF_CART, null);
        if (json == null) {
            return new ArrayList<>();
        }
        Type type = new TypeToken<List<CartItem>>() {}.getType();
        List<CartItem> items = gson.fromJson(json, type);
        return items != null ? items : new ArrayList<>();
    }

    public void addItem(Equipment equipment, int quantity) {
        List<CartItem> items = getItems();
        for (CartItem item : items) {
            if (item.getEquipmentId().equals(equipment.getId())) {
                item.setQuantity(item.getQuantity() + quantity);
                saveItems(items);
                return;
            }
        }
        items.add(new CartItem(equipment.getId(), equipment.getName(), quantity));
        saveItems(items);
    }

    public void removeItem(String equipmentId) {
        List<CartItem> items = getItems();
        for (int i = items.size() - 1; i >= 0; i--) {
            if (items.get(i).getEquipmentId().equals(equipmentId)) {
                items.remove(i);
            }
        }
        saveItems(items);
    }

    public void clear() {
        prefs.edit().remove(PREF_CART).apply();
    }

    public boolean isInCart(String equipmentId) {
        List<CartItem> items = getItems();
        for (CartItem item : items) {
            if (item.getEquipmentId().equals(equipmentId)) {
                return true;
            }
        }
        return false;
    }

    private void saveItems(List<CartItem> items) {
        prefs.edit().putString(PREF_CART, gson.toJson(items)).apply();
    }
}
