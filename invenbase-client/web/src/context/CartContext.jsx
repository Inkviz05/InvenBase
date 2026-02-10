import React, { createContext, useContext, useState, useEffect } from 'react';

const CartContext = createContext(null);

export const useCart = () => {
  const context = useContext(CartContext);
  if (!context) {
    throw new Error('useCart must be used within CartProvider');
  }
  return context;
};

export const CartProvider = ({ children }) => {
  const [cartItems, setCartItems] = useState([]);

  // Загружаем корзину из localStorage при инициализации
  useEffect(() => {
    const savedCart = localStorage.getItem('bookingCart');
    if (savedCart) {
      try {
        setCartItems(JSON.parse(savedCart));
      } catch (e) {
        console.error('Error loading cart from localStorage:', e);
      }
    }
  }, []);

  // Сохраняем корзину в localStorage при изменении
  useEffect(() => {
    localStorage.setItem('bookingCart', JSON.stringify(cartItems));
  }, [cartItems]);

  const addToCart = (equipment, quantity = 1) => {
    setCartItems(prevItems => {
      const existingItem = prevItems.find(item => item.equipment.id === equipment.id);
      
      if (existingItem) {
        // Если оборудование уже в корзине, обновляем количество
        const newQuantity = existingItem.quantity + quantity;
        const maxQuantity = equipment.available_quantity || equipment.quantity;
        
        return prevItems.map(item =>
          item.equipment.id === equipment.id
            ? { ...item, quantity: Math.min(newQuantity, maxQuantity) }
            : item
        );
      } else {
        // Добавляем новое оборудование
        return [...prevItems, { equipment, quantity: Math.min(quantity, equipment.available_quantity || equipment.quantity) }];
      }
    });
  };

  const removeFromCart = (equipmentId) => {
    setCartItems(prevItems => prevItems.filter(item => item.equipment.id !== equipmentId));
  };

  const updateQuantity = (equipmentId, quantity) => {
    if (quantity <= 0) {
      removeFromCart(equipmentId);
      return;
    }
    
    setCartItems(prevItems =>
      prevItems.map(item => {
        if (item.equipment.id === equipmentId) {
          const maxQuantity = item.equipment.available_quantity || item.equipment.quantity;
          return { ...item, quantity: Math.min(quantity, maxQuantity) };
        }
        return item;
      })
    );
  };

  const clearCart = () => {
    setCartItems([]);
  };

  const getCartCount = () => {
    return cartItems.reduce((sum, item) => sum + item.quantity, 0);
  };

  const getCartItemsCount = () => {
    return cartItems.length;
  };

  const isInCart = (equipmentId) => {
    return cartItems.some(item => item.equipment.id === equipmentId);
  };

  const value = {
    cartItems,
    addToCart,
    removeFromCart,
    updateQuantity,
    clearCart,
    getCartCount,
    getCartItemsCount,
    isInCart,
  };

  return <CartContext.Provider value={value}>{children}</CartContext.Provider>;
};

