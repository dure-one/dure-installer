package app.dure.installer;

import android.app.Application;
import android.util.Log;

/**
 * Custom Application class for Dure Installer
 *
 * This class is initialized before any activities are created and provides
 * a central point for application-level initialization.
 */
public class DureInstallerApplication extends Application {
    private static final String TAG = "DureInstallerApp";
    private static DureInstallerApplication instance;

    @Override
    public void onCreate() {
        super.onCreate();
        instance = this;

        Log.i(TAG, "Dure Installer Application initialized");
        Log.i(TAG, "Package: " + getPackageName());
    }

    public static DureInstallerApplication getInstance() {
        return instance;
    }
}
