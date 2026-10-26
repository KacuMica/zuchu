/*
 * zuchu-stat — tiny helper of Zuchu (C, in the spirit of kacu-sysinfo).
 * Reads /proc/stat and /proc/meminfo, prints a short line for the statusline:
 *   cpu=12% ram=43%
 *
 * Build: cc -O2 -o zuchu-stat zuchu-stat.c
 */

#include <stdio.h>
#include <string.h>

int main(void)
{
    FILE *f;
    char line[256];
    unsigned long long user, nice, sys, idle, total, load, memtotal = 0, memfree = 0, avail = 0;

    f = fopen("/proc/stat", "r");
    if (f) {
        if (fgets(line, sizeof line, f) &&
            sscanf(line, "cpu %llu %llu %llu %llu", &user, &nice, &sys, &idle) == 4) {
            total = user + nice + sys + idle;
            load = total ? (total - idle) * 100 / total : 0;
            printf("cpu=%llu%%", load);
        }
        fclose(f);
    }

    f = fopen("/proc/meminfo", "r");
    if (f) {
        while (fgets(line, sizeof line, f)) {
            if (strncmp(line, "MemTotal:", 9) == 0)
                sscanf(line + 9, "%llu", &memtotal);
            else if (strncmp(line, "MemAvailable:", 13) == 0)
                sscanf(line + 13, "%llu", &avail);
            else if (strncmp(line, "MemFree:", 8) == 0)
                sscanf(line + 8, "%llu", &memfree);
        }
        fclose(f);
        if (memtotal > 0) {
            unsigned long long used = memtotal - (avail ? avail : memfree);
            printf(" ram=%llu%%", used * 100 / memtotal);
        }
    }

    printf("\n");
    return 0;
}
