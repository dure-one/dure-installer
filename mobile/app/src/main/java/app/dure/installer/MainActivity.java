package app.dure.installer;

import android.app.NativeActivity;
import android.os.Bundle;
import androidx.core.view.WindowCompat;

/**
 * Custom NativeActivity that forces the window to respect system bars.
 *
 * This Activity extends android.app.NativeActivity and disables edge-to-edge mode
 * by calling WindowCompat.setDecorFitsSystemWindows(window, true), which tells
 * the system to layout content within the safe area (below status bar, above navigation bar).
 */
public class MainActivity extends NativeActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // Force the window to respect system bars and NOT draw under them
        // This tells Android to automatically inset the content area
        WindowCompat.setDecorFitsSystemWindows(getWindow(), true);
    }
}
