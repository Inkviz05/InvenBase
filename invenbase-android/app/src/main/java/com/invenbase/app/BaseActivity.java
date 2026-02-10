package com.invenbase.app;

import android.graphics.drawable.ColorDrawable;
import android.os.Bundle;
import android.view.View;
import android.view.ViewGroup;
import android.view.Window;

import androidx.appcompat.app.ActionBar;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.content.ContextCompat;

/**
 * Базовая активность: без заголовка, тёмный фон, скрытие панели/логотипа.
 */
public abstract class BaseActivity extends AppCompatActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        supportRequestWindowFeature(Window.FEATURE_NO_TITLE);
        super.onCreate(savedInstanceState);
        if (getWindow() != null) {
            getWindow().setBackgroundDrawable(new ColorDrawable(ContextCompat.getColor(this, R.color.background)));
        }
    }

    @Override
    protected void onPostCreate(Bundle savedInstanceState) {
        super.onPostCreate(savedInstanceState);
        hideActionBarLogo();
        hideDecorActionBar();
    }

    protected void hideActionBarLogo() {
        ActionBar ab = getSupportActionBar();
        if (ab != null) {
            ab.setDisplayShowHomeEnabled(false);
            ab.setDisplayUseLogoEnabled(false);
            ab.setLogo(null);
            ab.setIcon(null);
        }
    }

    /** Скрывает панель в декораторе окна (убирает белый квадрат, если он рисуется системой). */
    private void hideDecorActionBar() {
        try {
            if (getWindow() == null) return;
            View decor = getWindow().getDecorView();
            if (decor instanceof ViewGroup) {
                hideActionBarInView((ViewGroup) decor);
            }
        } catch (Throwable ignored) {
            // Не даём обходу по иерархии view вызвать краш приложения
        }
    }

    private void hideActionBarInView(ViewGroup root) {
        try {
            for (int i = 0; i < root.getChildCount(); i++) {
                View child = root.getChildAt(i);
                if (child == null) continue;
                String name = child.getClass().getSimpleName();
                // Скрываем только системные панель/тулбар по классу — не трогаем view по id/имени,
                // иначе на MIUI и др. может скрываться контент экрана (форма входа)
                if ("ActionBarContainer".equals(name) || "Toolbar".equals(name)) {
                    child.setVisibility(View.GONE);
                    // не return — в дереве может быть несколько таких view
                }
                if (child instanceof ViewGroup) {
                    hideActionBarInView((ViewGroup) child);
                }
            }
        } catch (Throwable ignored) { }
    }
}
