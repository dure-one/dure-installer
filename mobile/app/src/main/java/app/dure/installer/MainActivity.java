package app.dure.installer;

import android.app.NativeActivity;
import android.os.Bundle;
import android.view.View;
import android.view.WindowManager;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.graphics.Insets;

/**
 * Custom NativeActivity that applies window insets to avoid drawing under the status bar.
 *
 * This Activity extends android.app.NativeActivity and applies padding to the root view
 * based on system bar insets (status bar, navigation bar) to prevent content from being
 * obscured by system UI elements.
 */
public class MainActivity extends NativeActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // Apply window insets to the root view after the native content is created
        // We need to wait for the view hierarchy to be ready
        getWindow().getDecorView().post(() -> {
            applyWindowInsets();
        });
    }

    /**
     * Apply window insets to the root view to prevent content from drawing under system bars.
     */
    private void applyWindowInsets() {
        View rootView = getWindow().getDecorView();

        ViewCompat.setOnApplyWindowInsetsListener(rootView, (view, windowInsets) -> {
            // Get system bar insets (status bar + navigation bar)
            Insets insets = windowInsets.getInsets(WindowInsetsCompat.Type.systemBars());

            // Apply padding to push content below the status bar and above navigation bar
            view.setPadding(
                insets.left,
                insets.top,
                insets.right,
                insets.bottom
            );

            // Return the insets so they can be consumed by child views if needed
            return windowInsets;
        });

        // Request to apply insets
        ViewCompat.requestApplyInsets(rootView);
    }
}
