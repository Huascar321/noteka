#import <Foundation/Foundation.h>
#import <CoreLocation/CoreLocation.h>

typedef void (*LocationCallback)(void *, const char *);
static NSMutableSet *requests;
@interface NotekaLocation : NSObject <CLLocationManagerDelegate>
@property(nonatomic, strong) CLLocationManager *manager;
@property(nonatomic, strong) CLGeocoder *geocoder;
@property(nonatomic, assign) void *context;
@property(nonatomic, assign) LocationCallback callback;
@property(nonatomic, assign) BOOL finished;
@property(nonatomic, assign) BOOL requested;
@property(nonatomic, assign) BOOL hasLocation;
@end
@implementation NotekaLocation
- (void)finish:(NSDictionary *)result {
    if (self.finished) return;
    self.finished = YES;
    [self.manager stopUpdatingLocation];
    [self.geocoder cancelGeocode];
    NSData *data = [NSJSONSerialization dataWithJSONObject:result options:0 error:nil];
    NSString *json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
    self.callback(self.context, json.UTF8String ?: "{\"error\":\"Location unavailable\"}");
    [requests removeObject:self];
}
- (void)locationManagerDidChangeAuthorization:(CLLocationManager *)manager {
    CLAuthorizationStatus status = manager.authorizationStatus;
    if (status == kCLAuthorizationStatusDenied || status == kCLAuthorizationStatusRestricted) {
        [self finish:@{ @"error": @"Location permission denied or restricted. Enable Location Services in System Settings." }];
    } else if (status == kCLAuthorizationStatusAuthorizedAlways) {
        if (!self.requested) {
            self.requested = YES;
            [manager requestLocation];
            dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 25 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
                if (!self.hasLocation) [self finish:@{ @"error": @"Could not determine your location. Check Location Services and try again." }];
            });
        }
    }
}
- (void)locationManager:(CLLocationManager *)manager didChangeAuthorizationStatus:(CLAuthorizationStatus)status {
    (void)status;
    [self locationManagerDidChangeAuthorization:manager];
}
- (void)locationManager:(CLLocationManager *)manager didFailWithError:(NSError *)error {
    (void)manager; (void)error;
    [self finish:@{ @"error": @"Location unavailable. Check Location Services and try again." }];
}
- (void)locationManager:(CLLocationManager *)manager didUpdateLocations:(NSArray<CLLocation *> *)locations {
    (void)manager;
    CLLocation *location = locations.lastObject;
    if (!location || location.horizontalAccuracy < 0) { [self finish:@{ @"error": @"Location unavailable. Check Location Services and try again." }]; return; }
    if (self.hasLocation) return;
    self.hasLocation = YES;
    double lat = location.coordinate.latitude, lon = location.coordinate.longitude;
    NSDictionary *fallback = @{ @"latitude": @(lat), @"longitude": @(lon), @"name": @"Current location", @"admin1": @"", @"country": @"" };
    self.geocoder = [[CLGeocoder alloc] init];
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 5 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{ [self finish:fallback]; });
    [self.geocoder reverseGeocodeLocation:location completionHandler:^(NSArray<CLPlacemark *> *places, NSError *error) {
        (void)error;
        CLPlacemark *place = places.firstObject;
        NSString *name = place.locality ?: place.subAdministrativeArea ?: @"Current location";
        [self finish:@{ @"latitude": @(lat), @"longitude": @(lon), @"name": name, @"admin1": place.administrativeArea ?: @"", @"country": place.country ?: @"" }];
    }];
}
@end
void noteka_current_location(void *context, LocationCallback callback) {
    dispatch_async(dispatch_get_main_queue(), ^{
        if (!requests) requests = [NSMutableSet set];
        NotekaLocation *request = [NotekaLocation new];
        request.context = context; request.callback = callback;
        request.manager = [CLLocationManager new];
        request.manager.delegate = request;
        request.manager.desiredAccuracy = kCLLocationAccuracyKilometer;
        [requests addObject:request];
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 90 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
            if (!request.requested) [request finish:@{ @"error": @"Waiting for location permission timed out. Check the macOS permission prompt or Location Services in System Settings." }];
        });
        if (![CLLocationManager locationServicesEnabled]) { [request finish:@{ @"error": @"Location Services are disabled. Enable them in System Settings." }]; return; }
        CLAuthorizationStatus status = request.manager.authorizationStatus;
        if (status == kCLAuthorizationStatusNotDetermined) [request.manager requestWhenInUseAuthorization];
        else [request locationManagerDidChangeAuthorization:request.manager];
    });
}
